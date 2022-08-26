
use bitflags::bitflags;
use crate::client::*;

/// Basic structure which represents a Bot inside the library
/// Not the same as a discord bot!
pub struct Bot {

    /// The discord token for the bot utilised for connecting & accessing the discord api.
    pub token: String,

    /// Bitflags which allow for the selection of what gateway events to recieve, Some are privileged and require being toggled on in the developer page.
    /// https://discord.com/developers/docs/topics/gateway#list-of-intents
    pub intents: Intents,
}

impl Bot {

    /// Create a new [`Bot`] requiring basic fields set at initialisation
    pub fn new(token: String) -> Self {
        Self {
            token,
            intents: Intents::empty(),
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

    /// Main execution for a [`Bot`], initialisation and spawning of a [`DiscordClient`] & Related processes occur.
    pub fn elevate(self) {

    }

}

/// Enum for the different options available for sharding when the [`Bot`] is ran.
/// Only Shard 0 will recieve DMs, 
pub enum ShardingOptions {

    /// Automatically sets up sharding based on information about Session Start Limits.
    Automatic,

    /// Force [`Bot`] to utilise a set amount of shards
    SetAmount(u32),
}

bitflags::bitflags! {

    /// Bitflags Struct which represents all the possible intents for the Gateway Identify handshake event.
    /// https://discord.com/developers/docs/topics/gateway#list-of-intents
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