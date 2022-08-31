//! A file specifically designated to creation of structs which represent Payloads & Objects being sent through the Discord Gateway & related.
use std::{rc::Rc, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use anyhow::{Result, Context};

#[derive(Deserialize, Debug)]
/// [The limits imposed on new sessions which are started.][https://discord.com/developers/docs/topics/gateway#session-start-limit-object]
pub struct SessionStartLimit {

    #[serde(rename = "total")]
    /// The total number of session starts the current [`Bot`] is allowed
    pub total_sessions: u32,

    #[serde(rename = "remaining")]
    /// The remaining number of session starts the current [`Bot`] is allowed
    pub remaining_sessions: u32,

    /// The number of milliseconds after which the limit for starting sessions resets
    pub reset_after: u32,

    /// The number of identify requests allowed per 5 seconds
    /// Useful for determining the size for [shard buckets][https://discord.com/developers/docs/topics/gateway#sharding-max-concurrency]
    pub max_concurrency: u32,
}


#[derive(Deserialize, Debug)]
/// A struct which represents the response from the GetGatewayBot request
pub struct GetGatewayBotResponse {

    /// The WSS URL that can be used for connecting to the gateway
    pub url: String,

    /// The recommended number of shards to use when connecting
    pub shards: u32, 

    /// Information on the current session start limit
    pub session_start_limit: SessionStartLimit,
    
}

#[derive(Deserialize, Debug, Serialize, Clone)]
/// The Hello payload recieved from Discords gateway whenever a new connection gateway
pub struct Hello {
    /// The interval (in milliseconds) the client should heartbeat with
    pub heartbeat_interval: u32,
}

#[derive(Debug, Serialize, Clone)]
/// Payload used to trigger the initial handshake with the gateway.
pub struct Identify {

    
    /// The authentication token for this bot
    pub token: Arc<str>,

    #[serde(rename = "properties")]
    /// Information about the connection sent to discord.
    pub connection_properties: IdentifyConnectionProperties,

    // If a guild has more total members than the threshold then the gateway will stop sending offline members list of guild members.
    //pub offline_member_threshold: u8,

    /// The current shard which is an array of [shard_id, total_shards]
    pub shard: [u32; 2],

    //Compress not included as currently this library does not and wont ever support packet compression
    //pub initial_presence: 

    /// the Gateway intents you wish to recieve
    pub intents: u32,
}

#[derive(Debug, Serialize, Clone, Copy)]
/// Connection information/properties related to the Identify handshake payload.
pub struct IdentifyConnectionProperties {

    #[serde(rename = "os")]
    /// The operating system the bot uses
    pub operating_system: &'static str,

    /// Library Name
    pub browser: &'static str,

    /// Library Name
    pub device: &'static str,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
/// Representation of a payload to be sent or recieved from Discords Gateway.
pub struct Payload<T> {

    #[serde(rename = "op")]
    /// The opcode of this payload that denotes the payload type
    pub opcode: u32,

    #[serde(rename = "d")]
    /// The data of this payload
    pub data: T,

    #[serde(rename = "s")]
    /// Sequence number of this payload 
    /// Used for resuming sessions and heartbeats
    pub sequence_number: Option<u32>,   

    #[serde(rename = "t")]
    /// The event name for this payload
    pub event_name: Option<String>,  
}

impl<T: Serialize + Send + 'static> Payload<T> {

    /// Converts a struct into a Tungstenite Message through serde_json
    pub async fn serialize_to_message(self) -> Result<Message> {

        tokio::task::spawn_blocking(move || -> Result<Message> {

            // Serialize the struct by converting it into a string & then create a Message from that.
            Ok(Message::Text(serde_json::to_string(&self)
                .context("Failed to Serialize payload into Message for sending.")?))

        }).await?
    }
}

impl<T> Payload<T> {

    /// Creates a new payload with only the opcode & data fields set.
    pub fn new(opcode: u32, data: T) -> Self {
        Self {
            opcode,
            data,
            sequence_number: None,
            event_name: None,
        }
    }
}
