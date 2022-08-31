


use std::sync::Arc;

use bitflags;
use reqwest::{Method};
use crate::http::*;
use crate::gateway_structs::GetGatewayBotResponse;
use crate::{BASE_API_URL, DISCORD_API_VERSION};
use crate::websocket::*;
use tokio::sync::*;
use anyhow::{Result, Context};

/// Basic structure which represents a Bot inside the library
pub struct Bot {        

    /// The discord token for the bot utilised for connecting & accessing the discord api.
    pub token:  Arc<str>,

    /// [Bitflags which allow for the selection of what gateway events to recieve][https://discord.com/developers/docs/topics/gateway#list-of-intents]
    /// Some are privileged and require being toggled on in the developer page.
    pub intents: Intents,

    /// Enum option which determines if automatic sharding should be used or if shards should be created based on a set amount.
    pub sharding_option: ShardingOption,
           
}

impl Bot {          

    /// Create a new [`Bot`] requiring basic fields set at initialisation
    pub fn new(token: String) -> Self {
        Self {
            token: token.into(),
            intents: Intents::empty(),
            sharding_option: ShardingOption::Automatic,
        }
    }

    /// Sets the intents of a [`Bot`] to be the union between the existing [`Intents`] in the bot and another [`Intents`]
    pub fn union_intents(&mut self, intents: Intents) {
        self.intents = self.intents.union(intents);
    }

    /// Sets the intents of a [`Bot`] to be the intersection between the existing [`Intents`] in the bot and another [`Intents`]
    pub fn intersection_intents(&mut self, intents: Intents) {
        self.intents = self.intents.intersection(intents);
    }

    /// Inserts or removes the specified [`Intents`] for a [`Bot`] depending on the passed [`Intents`] and the value [`bool`]
    pub fn set_intents(&mut self, intents: Intents, value: bool) {
        self.intents.set(intents, value);
    }

    /// Main execution for a [`Bot`] and initialisation of a [`DiscordClient`]
    /// Establish a connection to the Discord Gateway & start listening to the events.
    pub async fn elevate(self) -> Result<()> {

        // Create the DiscordHttpClient to be able to request data from the Discord Api.
        let http_client = DiscordHttpClient::new(BASE_API_URL, DISCORD_API_VERSION, self.token.clone())
            .context("Failed to create DiscordHttpClient")?;
        

        // Setup the channel for Requests to the Discord api through the DiscordHttpClient
        let (http_channel_sender, http_channel_reciever) = mpsc::channel(50);

        // Spawn the DiscordHttpClientRequest processing channel.
        tokio::spawn(
            async move {
                http_client.handle_channel_inbound_requests(http_channel_reciever).await
            }
        );

        // Cloning is an acceptable operation as Sender contains an arc so this acts as just cloning a pointer to the sender
        let client_sender = http_channel_sender.clone();

        // Send a GET request with path gateway/bot/ so we can get information on connecting to the gateway & sharding
        // Also serves as a way to check if the token is valid or not etc. 
        let gateway_bot_response : GetGatewayBotResponse = DiscordHttpRequest::new(DiscordHttpReqType::GetGatewayBot, Method::GET).request(client_sender)
            .await    
            .context("Failed in retrieving gateway/bot required for starting up discord Gateway connection.")?; 

        // Create a sharded [`DiscordGatewayClient`]
        let _gateway_client = DiscordGatewayClient::new_with_shards(&self, gateway_bot_response).await
            .context("Failed to create DiscordGatewayClient")?;
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;
        Ok(())
        
    }

}

/// Enum for the different options available for sharding when the [`Bot`] is ran.
/// Only Shard 0 will recieve DMs, 
pub enum ShardingOption {

    /// Automatically sets up sharding based on information about Session Start Limits.
    Automatic,

    /// Force [`Bot`] to utilise a set amount of shards
    SetAmount(u32),
}


bitflags::bitflags! {

    /// [Bitflags Struct which represents all the possible intents for the Gateway Identify handshake event.][https://discord.com/developers/docs/topics/gateway#list-of-intents]
    pub struct Intents: u32 {
        
        const GUILDS = 1 << 0;

        /// This is a privileged [`Intent`]
        const GUILD_MEMBERS = 1 << 1;
        const GUILD_BANS = 1 << 2;
        const GUILD_EMOJIS_AND_STICKERS = 1 << 3;
        const GUILD_INTEGRATIONS = 1 << 4;
        const GUILD_WEBHOOKS = 1 << 5;
        const GUILD_INVITES = 1 << 6;
        const GUILD_VOICE_STATES = 1 << 7;

        /// This is a privileged [`Intent`]
        const GUILD_PRESENCES = 1 << 8;
        const GUILD_MESSAGES = 1 << 9;
        const GUILD_MESSAGE_REACTIONS = 1 << 10;
        const GUILD_MESSAGE_TYPING = 1 << 11;
        const DIRECT_MESSAGES = 1 << 12;
        const DIRECT_MESSAGE_REACTIONS = 1 << 13;
        const DIRECT_MESSAGE_TYPING = 1 << 14;

        /// This is a privileged [`Intent`]
        const MESSAGE_CONTENT = 1 << 15;
        const GUILD_SCHEDULED_EVENTS = 1 << 16;
        const AUTO_MODERATION_CONFIGURATION = 1 << 20;
        const AUTO_MODERATION_EXECUTION = 1 << 21;

    }

}