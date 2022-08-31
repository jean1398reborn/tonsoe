use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use futures_util::StreamExt;
use serde::Serialize;


use tokio::sync::mpsc::{Sender as GatewaySinkSender};
use tokio::sync::broadcast::{Sender as GatewayStreamSender};
use tokio::time::*;
use crate::gateway_structs::{Payload, Identify};
use crate::websocket::ReadSplitStream;

#[derive(Clone)]
/// Contains information on a connection to the discord gateway.
/// Contains the corresponding [`GatewaySinkSender`] and [`GatewayStreamReciever`] for the connection.
pub struct Gateway { 

    /// The sender to a corresponding shards channel which processes the [`GatewayCommand`] and sends it through the gateway.
    pub gateway_sink_sender: GatewaySinkSender<GatewayCommand>,

    /// The sender for recieving events from the discord gateway. utilised for subscribing to create new recievers.
    pub gateway_stream_sender: GatewayStreamSender<GatewayEvent>,

    /// Information about what shard this gateway connection belongs to. & heartbeating
    pub connection_id: GatewayConnectionIdentifier,



}

#[derive(Clone)]
/// Struct which represents information about a connection to the Discord gateway.
/// Contains information which could be specific to a certain shard & heartbeating
pub struct GatewayConnectionIdentifier {
    /// The id for the shard which this [`Gateway`] represents.
    /// This is zero-based (starts at 0 for the first shard.)
    pub shard_id: u32,

    /// The total amount of shards present.
    /// This is not zero-based, e.g: three shards would have a shard_total of 3.
    pub shard_total: u32,

    /// The interval (in milliseconds) the client should heartbeat with
    pub heartbeat_interval: u32,
    
    /// Number which is the last sequence number recieved from discords Gateway
    pub sequence_identifier: Arc<AtomicU32>

}

#[derive(Clone, Debug, Serialize)]
/// Commands a [`Gateway`] channel can process 
pub enum GatewayCommand {
    Heartbeat(Payload<u32>),
    Identify(Payload<Identify>),
}

#[derive(Clone)]
/// Responses from a [`Gateway`] channel.
pub enum GatewayEvent {

}
  
impl Gateway {

    ///Send heartbeats through this current shard to keep it alive.
    pub async fn heartbeat(self) {
        let heartbeat_interval = Duration::from_millis(self.connection_id.heartbeat_interval as u64);

        // First heartbeat happens after heartbeat_interval * jitter
        // Jitter is a random value between 0 and 1.
        let jitter = rand::random::<f32>();

        // Sleep for the first heartbeat
        sleep(Duration::mul_f32(heartbeat_interval, jitter)).await;

        loop {

            // Create the payload
            let previous_sequence_number = self.connection_id.sequence_identifier.load(Ordering::Acquire);
            let heartbeat_payload = Payload::new(1, previous_sequence_number);

            // Send it
            self.gateway_sink_sender.send(GatewayCommand::Heartbeat(heartbeat_payload)).await
                .expect("Unable to send Heartbeat Payload through sink channel!");

            // Wait for next heartbeat
            sleep(heartbeat_interval).await;
        }
    } 

    pub async fn recieve_gateway_events(self, mut read_stream: ReadSplitStream) {

        // Recieve next payload
        while let _payload = read_stream.next().await {

        }
    }
}