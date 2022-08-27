//! A file specifically designated to creation of structs which represent Payloads & Objects being sent through the Discord Gateway & related.
use serde::Deserialize;

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