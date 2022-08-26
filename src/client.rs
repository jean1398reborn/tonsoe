use std::collections::HashMap;
use crate::bot::*;
use crate::gateway::*;

/// Represents the client which acts as a connection between the Discord api & a [`Bot`]
pub struct DiscordClient {

    /// A [`HashMap`] of all existing gateway connections where key: shard_id & value: Gateway 
    pub shards: HashMap<u32, Gateway>,

    /// The [`Bot`] accociated with this client.
    pub bot: Bot

}