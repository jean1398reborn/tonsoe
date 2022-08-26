//! Example showing most basic bot functionality of coming online.
// Not Complete!

use tonsoe::bot::*;
#[tokio::main]
async fn main() {

    // Grab the token from https://discord.com/developers
    // Note: You need to get this before the BotBuilder is initialised as it cannot be changed after.
    let token: String = String::from("Token");

    // Create a new bot
    let mut bot = Bot::new(token);

    // Set any intents you wish by creating an Intents and utilising the set_intents method.
    let bot_intents = Intents::from(
        Intents::GUILDS | Intents::DIRECT_MESSAGES | Intents::GUILD_MESSAGES
    );

    // Set the intents given by bot_intents to be true
    bot.set_intents(bot_intents, true);

    // Execute the bot.
    bot.elevate().await;
}
