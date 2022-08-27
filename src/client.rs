
use reqwest::{Client, Url, Method, Response as HttpResponse, RequestBuilder};
use reqwest::header::*;

use std::collections::HashMap;
use std::str::FromStr;

use tokio::sync::mpsc::{Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{Sender as OneshotSender};
use tokio::sync::*;

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

#[derive(Debug)]
pub enum DiscordHttpReqType {
    /// Retrieves information on connecting the Discord [`Gateway`] and additional metadata for sharding bots.
    GetGatewayBot
}

#[derive(Debug)]
/// Represents a request to the DiscordHttpClient channel.
pub struct DiscordHttpClientRequest {

    /// The type of request which serves as what to request from the api
    pub request: DiscordHttpRequest,

    /// The oneshot channel where the response for this request should be sent to.
    pub response_sender: OneshotSender<HttpResponse>,
    
}

#[derive(Debug)]
/// Represents a request to the Discord Api
pub struct DiscordHttpRequest {

    /// The type of request which serves as what to request from the api
    pub request_type: DiscordHttpReqType,

    /// The method to use for the request
    pub method: Method,

    /// A map of extra headers to add upon the default ones.
    pub headers: HeaderMap,
    
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
            DiscordHttpReqType::GetGatewayBot => "gateway/bot",
        }
    }
}

impl DiscordHttpClient {

    pub fn new(base_url: &'static str, version: u32, token: String) -> Self {

        // Default headers required for utilising discord api.
        let mut default_headers = HeaderMap::new();

        // Add authorization to the default headers as every request which involves the bot requires this.
        let authorization_header_value = HeaderValue::from_str(&format!("Bot {token}"))
            .expect("Authorization header value should've been created from format! macro with bot token");
        default_headers.append("Authorization", authorization_header_value);


        // Create the reqwest client utilised for https requests to discords api.
        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .expect("Should've created reqwest DiscordHttpClient with default headers");

        // Create the base_uri utilised for all requests once here.
        let url = Url::from_str(&format!("{base_url}/v{version}/"))
            .expect("URL should be created by combining base_url with version through format! macro");
        
        Self {
            client,
            base_url: url,
        }
    }

    /// Sets up a request through [`DiscordHttpClient`]
    /// The [`Method`], [Path][`Url`] and [Headers][`HeaderMap`] are retrieved through a [`DiscordHttpRequest`]
    pub async fn request(&self, request: DiscordHttpRequest) ->  RequestBuilder {

        // Constructs the full URL utilised for this request
        let url_address = self.base_url.join(request.get_request_path())
            .expect("URL_Address should be created by joining base_url with the requests path");

        // Create the Request and send it retrieving the result of the request.
        self.client.request(request.method, url_address)
            .headers(request.headers)
    }

    /// Method which acts as the processor for recieving inbound a [`DiscordHttpRequest`] to the [`DiscordHttpClient`] channel.
    /// The inbound channel is [multi-producer single-consumer][`MspcReceiver`]
    /// The response channel is [oneshot][`OneshotSender`]
    pub async fn handle_channel_inbound_requests(self, mut reciever: MpscReceiver<DiscordHttpClientRequest>) {

        // Recieve new inbound requests from the reciever
        while let Some(request) = reciever.recv().await {

            // Create our request to send.
            let request_builder = self.request(request.request).await;

            // Spawn a new task so we can handle sending the request asynchronously so it doesnt block this channel.
            tokio::spawn(
                async move {
                    
                    let response = request_builder.send().await
                        .expect("Attempted to send request to Discord Api");

                    // Get the StatusCode of the request
                    let status = response.status();

                    // Check for any issues which can be identified through the status code 
                    let response = match status {
                        reqwest::StatusCode::UNAUTHORIZED => {
                            panic!("Error {:?}. Perhaps your token is invalid?", response.error_for_status())
                        },
                        _ => {
                            response
                        },
                    };

                    // Send back the response, errors are ignored as dropping the reciever is acceptable when it no longer wants the response.
                    let _ = request.response_sender.send(response);
                }
            );

        }

    }
}

/// Simplifies sending process for sending a [`DiscordHttpRequest`] through the [`DiscordHttpClient`] mspc request processing channel.
pub async fn send_discord_http_request(request: DiscordHttpRequest, http_client_sender: MpscSender<DiscordHttpClientRequest>) -> HttpResponse {

    // Create the channel to recieve the response from the DiscordHttpClient channel.
    let (response_sender, response_reciever) = oneshot::channel();

    // Create the Request struct to send through the channel
    let client_request = DiscordHttpClientRequest { 
        response_sender: response_sender, 
        request: request
    };

    // Send it through the request processing channel
    http_client_sender.send(client_request)
        .await
        .expect("Attempted to send Request through DiscordHttpClient mpsc channel");

    // Recieve the response from the Request processor
    response_reciever.await
        .expect("Attempted to recieve response from DiscordHttpClient channel processor")
    
}