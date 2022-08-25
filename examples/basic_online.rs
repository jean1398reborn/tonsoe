//! Example showing most basic bot functionality of coming online.
// Not Complete!

use std::env;
use tonsoe::bot_client::*;

fn main() {

    // Grab the token from https://discord.com/developers
    // Note: You need to get this before the [`BotBuilder`] is initialised as it cannot be changed after.
    let token: String = String::from("Token");

    //Create a new Builder for our [`Bot`] using [`BotBuilder`]
    let bot = BotBuilder::new(token);

}
