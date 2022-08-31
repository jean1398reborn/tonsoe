//! ### What is Tonsoe?
//! Tonsoe is an open-source discord library created for and in Rust, with a focus on bot development.
//! The library is currently in an extremely experimental state where changes are made rapidly and will be breaking.

/// The base api url for Discord http requests.
pub const BASE_API_URL : &'static str = "https://discord.com/api";

/// Represents the version of the discord api utilised by the library
pub const DISCORD_API_VERSION: u32 = 10;

 
pub mod bot;
pub mod websocket;
pub mod gateway;
pub mod gateway_structs;
pub mod http;