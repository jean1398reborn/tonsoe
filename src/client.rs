
use reqwest::{Client, Url, Method, Response as HttpResponse, Error as HttpError};
use reqwest::header::*;

use std::collections::HashMap;
use std::str::FromStr;


use crate::bot::*;
use crate::gateway::*;

/// Represents the client which acts as a connection between the Discord api & a [`Bot`]
pub struct DiscordClient {

    /// A [`HashMap`] of all existing gateway connections where key: shard_id & value: Gateway 
    pub shards: HashMap<u32, Gateway>,

    /// The [`Bot`] accociated with this client.
    pub bot: Bot


}

pub struct DiscordHttpClient {

    // hyper HTTP client used for requests on the Discord api.
    pub client: Client,

    // The base url to be built upon when requesting.
    pub base_url: Url,

}

pub enum DiscordHttpReqType {
    /// Retrieves information on connecting the Discord [`Gateway`] and additional metadata for sharding bots.
    GetGatewayBot
}

/// Represents a request to the Discord Api
pub struct DiscordHttpRequest {

    /// The type of request which serves as what to request from the api
    pub request_type: DiscordHttpReqType,

    /// The method to use for the request
    pub method: Method,

    /// A map of extra headers to add upon the default ones.
    pub headers: HeaderMap
    
}

impl DiscordHttpRequest {

    /// Constructs a new [`DiscordHttpRequest`]
    pub fn new(request_type: DiscordHttpReqType, method: Method) -> Self {
        Self { 
            request_type, 
            method, 
            headers: HeaderMap::new() 
        }
    }

    /// Adds a header to the [`HeaderMap`] of the request.
    pub fn add_header(&mut self, header_key: &'static str, header_value: &String) -> Result<(), InvalidHeaderValue> {

        // Convert &String value into HeaderValue
        let header_value = HeaderValue::from_str(header_value)?; 

        // Append new header to existing map
        self.headers
            .append(header_key, header_value);

        Result::Ok(())
    }

    /// Retrieve the str extension to the base uri's path from the enum representation.
    pub fn get_request_path(&self) -> &'static str {
        match self.request_type {
            DiscordHttpReqType::GetGatewayBot => "/gateway/bot",
        }
    }
}

impl DiscordHttpClient {

    pub fn new(base_url: &'static str, version: u32, token: String) -> Self {

        // Default headers required for utilising discord api.
        let mut default_headers = HeaderMap::new();

        // Add authorization to the default headers as every request which involves the bot requires this.
        let authorization_header_value = HeaderValue::from_str(&format!("Bot {token}")).expect("Unable to create default Authorization header value");
        default_headers.append("Authorization", authorization_header_value);


        // Create the reqwest client utilised for https requests to discords api.
        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .expect("Failed to create DiscordHttpClient reqwest Client");

        // Create the base_uri utilised for all requests once here.
        let url = Url::from_str(&format!("{base_url}/v{version}"))
            .expect("Failed to create base_uri for DiscordHttpClient");
    
        Self {
            client,
            base_url: url,
        }
    }

    /// Sets up a request through the [`DiscordHttpClient`]
    /// The method, path and any arguments are retrieved through a [`DiscordHttpRequest`]
    pub async fn request(&self, request: DiscordHttpRequest) -> Result<HttpResponse, HttpError> {
    
        // Constructs the full URL utilised for this request
        let url_address = self.base_url.join(request.get_request_path())
            .expect("Unable to create full url_address for DiscordHttpRequest");

        // Create the Request and send it returning the result of the response.
        self.client.request(request.method, url_address)
        .headers(request.headers)
        .send()
        .await

    }
}
