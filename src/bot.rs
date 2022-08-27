
use bitflags;
use reqwest::Method;
use crate::client::*;
use crate::gateway_structs::GetGatewayBotResponse;
use crate::{BASE_API_URL, DISCORD_API_VERSION};
use tokio::sync::*;

/// Basic structure which represents a Bot inside the library
/// Not the same as a discord bot!
pub struct Bot {

    /// The discord token for the bot utilised for connecting & accessing the discord api.
    pub token: String,

    /// [Bitflags which allow for the selection of what gateway events to recieve][https://discord.com/developers/docs/topics/gateway#list-of-intents]
    /// Some are privileged and require being toggled on in the developer page.
    pub intents: Intents,

    /// Enum option which determines if automatic sharding should be used or if shards should be created based on a set amount.
    pub sharding_option: ShardingOption,
}

impl Default for Bot {
    fn default() -> Self {
        Self {
            token: String::new(),
            intents: Intents::empty(),
            sharding_option: ShardingOption::Automatic,
        }
    }
}

impl Bot {

    /// Create a new [`Bot`] requiring basic fields set at initialisation
    pub fn new(token: String) -> Self {
        Self {
            token,
            intents: Intents::empty(),
            ..Default::default()
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
    pub async fn elevate(self) {

        // Create the DiscordHttpClient to be able to request data from the Discord Api.
        let http_client = DiscordHttpClient::new(BASE_API_URL, DISCORD_API_VERSION, self.token);

        // Setup the channel for Requests to the Discord api through the DiscordHttpClient
        let (channel_sender, channel_reciever) = mpsc::channel(50);

        // Spawn the DiscordHttpClientRequest processing channel.
        tokio::spawn(
            async move {
                http_client.handle_channel_inbound_requests(channel_reciever).await
            }
        );

        // Cloning is an acceptable operation as Sender contains an arc so this acts as just cloning a pointer to the sender
        let client_sender = channel_sender.clone();

        // Send a GET request with path gateway/bot/ so we can get information on connecting to the gateway & sharding.
        let gateway_bot_response : GetGatewayBotResponse = send_discord_http_request(DiscordHttpRequest::new(DiscordHttpReqType::GetGatewayBot, Method::GET), client_sender)
            .await
            .json()
            .await
            .expect("Attempted to convert content of GetGatewayBot response into struct");
        
        println!("{:#?}", gateway_bot_response);
        
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