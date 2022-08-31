use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use crate::DISCORD_API_VERSION;
use crate::bot::*;
use crate::gateway::*;
use crate::gateway_structs::*;
use anyhow::Context;
use futures_util::SinkExt;
use futures_util::StreamExt;
use async_trait::async_trait;
use anyhow::Result;
use futures_util::stream::SplitSink;
use futures_util::stream::SplitStream;
use reqwest::Url;
use serde::de::*;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Sender as GatewaySinkSender, Receiver as GatewaySinkReceiver};
use tokio::sync::broadcast::{Sender as GatewayStreamSender, Receiver as GatewayStreamReciever};
use tokio::sync::*;
use tokio::task;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

/// An Arc RwLock hashmap utilised for accessing the different channels to created Gateways
pub type ShardMap = Arc<RwLock<HashMap<u32, Gateway>>>;

/// Type Alias which shortens the Split Stream from the Discord Gateway websocket.
pub type ReadSplitStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

/// Type Alias which shortens the Split Sin&k from the Discord Gateway websocket.
pub type WriteSplitSink = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

/// Represents the client which acts as a connection between the Discord Gateway & a [`Bot`]
pub struct DiscordGatewayClient {

    /// A [`HashMap`] of all existing gateway connections where key: shard_id & value: Sender for the gateway's channel.
    pub shards: ShardMap,

}

impl DiscordGatewayClient {

    /// Creates a new [`DiscordGatewayClient`] with sharded gateway connections using information provided by the GET gateway/bot request.
    pub async fn new_with_shards(bot: &Bot, gateway_bot_response: GetGatewayBotResponse) -> Result<Self> {

        // Get amount of shards to use when connecting to the Discord Gateway
        let shard_amount = match bot.sharding_option {

            // Shard amount is set based on the recommended amount from Discord
            ShardingOption::Automatic => {
                gateway_bot_response.shards
            },

            // Shard amount is set manually based on what the user desires.
            ShardingOption::SetAmount(amount) => {
                
                if amount == 0 {
                    return Err(anyhow::Error::msg("Amount of shards set manually must be > 0"))
                }

                amount
            },
        };

        // Limits for creating sessions
        let session_limits = gateway_bot_response.session_start_limit;

        // Get the url for the gateway we want the shards to connect to.
        let gateway_url = Url::from_str(&format!("{}/?v{}&encoding=json", gateway_bot_response.url, DISCORD_API_VERSION))
            .context("Failed to create Discord Gateway websocket URL")?;
        
        // Create the map for shards, this is wrapped in an Arc & a RwLock since we dont want to wait for ALL shards to be started before the bot can respond etc.
        let shard_map = Arc::new(RwLock::new(HashMap::new()));

        // A cloned Arc for the identify payloads token field.
        let token = bot.token.clone();

        // The name of the package
        let package_name = env!("CARGO_PKG_NAME");

        // The information about our connection sent to discord.
        let connection_properties = IdentifyConnectionProperties {
            operating_system: env::consts::OS,
            browser: package_name,
            device: package_name,
        };

        // Get the raw value of the intents stored within the bot to send to discord.
        let intents = bot.intents.bits();

        let shard_map_spawning   = shard_map.clone();
        // Create the shards, since there is a wait required we spawn this as a task so it doesnt block other functionality from existing shards which have already been created.
        tokio::spawn(
            async move {
                for shard_id in 0..shard_amount {
                    
                    // The payload to send to discord to handshake with the gateway for our bot
                    let shard_identify = Identify {
                        token: token.clone(),
                        connection_properties: connection_properties,
                        shard: [shard_id, shard_amount],
                        intents,
                    };
                    
                    // Convert it into a form where we can send it to discords gateway.
                    let identify_payload = Payload::new(2, shard_identify);

                    // Shards are made in set amounts called "buckets" the size of these buckets depends on max_concurrency under session_limits
                    // After a bucket amount of shards are created a 5 second wait is required
                    if shard_id % session_limits.max_concurrency == 0 && shard_id != 0 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await
                    }
                    
                    // Create the shard and add it to the map
                    shard_map_spawning.add_shard_to_map(shard_id, shard_amount, &gateway_url, identify_payload)
                        .await
                        .expect("Attempted to spawn shard");
                    
                }
            }
        );

        Ok(Self {
            shards: shard_map
        })
    }
}

#[async_trait]
/// Trait which exists so we can add a method which will add a new shard to a [`ShardMap`]
pub trait NewShardToMap {
    /// Method which adds a new shard to this [`ShardMap`]
    async fn add_shard_to_map(&self, shard_id: u32, total_shards: u32, gateway_url: &Url, identify_payload: Payload<Identify>) -> Result<()> ;
}

#[async_trait]
impl NewShardToMap for ShardMap {
    /// Creates & connects a new shard [`Gateway`] to the discord api and adds the [`Gateway`] to this [`ShardMap`]
    async fn add_shard_to_map(&self, shard_id: u32, shard_amount: u32, gateway_url: &Url, identify_payload: Payload<Identify>) -> Result<()> {

        // Create the sender and reciever utilised for the Gateway when recieving commands
        let (sink_channel_sender, mut sink_channel_reciever) = mpsc::channel(32);

        // Create the sender and reciever utilised for the Gateway when sending responses recieved from discord.
        let (stream_channel_sender, mut stream_channel_reciever) = broadcast::channel(32);

        println!("{:#?}", gateway_url.to_string());

        // Since we have to spawn them in order from 0 -> shard_amount we cant spawn them concurrently, utilises except as failing to create a shard can lead to catastrophic failure.
        let (websocket_stream, response) = tokio_tungstenite::connect_async(gateway_url).await
            .expect("Attempted to create Gateway Shard");

        // Split the stream up into a sink and a stream for channels.
        let (mut write_sink, mut read_stream) = websocket_stream.split();

        // Get the Hello payload send from discord.
        let hello_payload = read_stream.read_deserialize_next_payload::<Payload<Hello>>().await
            .context("Failed to recieve Hello Payload from Discord Gateway")?;

        // Identifier struct for our connection containing information on sharding & heartbeating
        let connection_identifier = GatewayConnectionIdentifier {
            shard_id,
            shard_total: shard_amount,
            heartbeat_interval: hello_payload.data.heartbeat_interval,
            sequence_identifier: Arc::new(AtomicU32::new(0))
        };

        // Send the identify payload through the sink which will send the handshake payload to the gateway 
        sink_channel_sender.send(GatewayCommand::Identify(identify_payload)).await
        .expect("Failed to send Identify Payload to sink channel.");

        // Create our Gateway utilised for communicating with the channels which send and recieve to Discords Gateway
        let gateway = Gateway {
            gateway_sink_sender:sink_channel_sender,
            gateway_stream_sender:stream_channel_sender, 
            connection_id: connection_identifier.clone(), 
        };

        // Spawn the processing function for sending values through the gateway.
        tokio::spawn(async move {
            process_gateway_send_commands(sink_channel_reciever, write_sink).await
        });

        let heartbeat_gateway = gateway.clone();
        // Spawn the function which will keep sending heartbeats to Discords gateway.
        tokio::spawn( async move {
            heartbeat_gateway.heartbeat().await
        });

        let receiving_gateway = gateway.clone();
        // Spawn the function which will recieve gateway events
        tokio::spawn( async move {
            receiving_gateway.recieve_gateway_events(read_stream).await
        });

        // Gain the write guard to our shard_map so we can add this Gateway to it.
        let mut shard_map_write = self.write().await;

        // Insert our gateway into the shard_map
        shard_map_write.insert(shard_id, gateway);

        Ok(())
    }
}

#[async_trait]
/// Trait which adds deserialization read methods to payloads recieved from the Discord Gateway
pub trait DeserializeRecievePayload {
    async fn read_deserialize_next_payload<T: DeserializeOwned + Send + 'static>(&mut self) -> Result<T>;
}

#[async_trait]
impl DeserializeRecievePayload for ReadSplitStream {
    /// Reads the next payload and attempts to deserialize it to type T, 
    /// fails if the next payload is unable to be deserialized to type T or Gateway was closed
    async fn read_deserialize_next_payload<T: DeserializeOwned + Send + 'static>(&mut self) -> Result<T> {

        // Recieve the next payload from the gateway
        let next_payload = self.next()
            .await
            .context("Gateway was closed when attempting to read next item")?
            .context("Failed in checking to see if Gateway was connected")?;

        // Convert the recieved payload message into &str type for deserialization
        let string_payload = next_payload.to_string();

        // Spawn a task so we can asynchronise the process
        tokio::task::spawn_blocking( move || -> Result<T> {
            
            // Deserialize the payload and return it
            serde_json::from_str::<T>(&string_payload)
                .context(format!("Failed to deserialize payload into type {:?}", std::any::type_name::<T>()))
        

        }).await?
    } 
}


/// Process a [`GatewayCommand`] send through the channel into a Message & send the message to Discords gateway.
pub async fn process_gateway_send_commands(mut sink_channel_reciever: GatewaySinkReceiver<GatewayCommand>, mut sink: WriteSplitSink) {
    
    // Create a new Mutex with Arc for concurrency with the split sink
    let sink = Arc::new(Mutex::new(sink));

    // Recieve new inbound gateway commands from the reciever
    while let Some(command) = sink_channel_reciever.recv().await {

        // Clone/create another Arc to the sink so we can send it into the tokio task for sending to discord.
        let send_sink = sink.clone();

        // Spawn a task to send the commands payload through the gateway
        task::spawn( async move {

            // Serialize the recieved payload into a message so we can send it through the sink
            let message_payload =  match command {
                GatewayCommand::Heartbeat(heartbeat_payload) => heartbeat_payload.serialize_to_message().await,
                GatewayCommand::Identify(identify_payload) => identify_payload.serialize_to_message().await,
            }.context("Failed to Serialize payload into Message").unwrap();

            // Send the message through the sink.
            send_sink
                .lock()
                .await
                .send(message_payload)
                .await
                .expect("Failed to send payload to Discord")

        });

    }

}