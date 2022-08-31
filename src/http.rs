
use reqwest::{Client, Url, Method, Response as HttpResponse, RequestBuilder};
use reqwest::header::*;
use serde::{de};


use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::mpsc::{Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{Sender as OneshotSender};
use tokio::sync::*;
use anyhow::{Result, Context};

/// Shortened Alias for Mpsc channel sender for a [`DiscordHttpClientRequest`]
pub type DiscordHttpClientReqSender = MpscSender<DiscordHttpClientRequest>;

pub enum DiscordApiError {
    Unauthorized
}
/// The client
pub struct DiscordHttpClient {

    // reqwest HTTP client used for requests on the Discord api.
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
    pub response_sender: OneshotSender<Result<HttpResponse>>,
    
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

    /// Sends a [`DiscordHttpRequest`] to a [`DiscordHttpClientReqSender`] to process the request
    pub async fn request<T: de::DeserializeOwned>(self, http_client_sender: DiscordHttpClientReqSender) -> Result<T> {
        Ok(send_discord_http_request(DiscordHttpRequest::new(DiscordHttpReqType::GetGatewayBot, Method::GET), http_client_sender)
            .await?
            .json::<T>()
            .await?)
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

    pub fn new(base_url: &'static str, version: u32, token: Arc<str>) -> Result<Self> {

        // Default headers required for utilising discord api.
        let mut default_headers = HeaderMap::new();

        // Add authorization to the default headers as every request which involves the bot requires this.
        let authorization_header_value = HeaderValue::from_str(&format!("Bot {token}"))
            .context("Failed to create default authorization header for Http Requests, perhaps token inputted is not ASCII?")?;
        default_headers.append("Authorization", authorization_header_value);


        // Create the reqwest client utilised for https requests to discords api.
        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .context("Should've created reqwest DiscordHttpClient with default headers")?;

        // Create the base_uri utilised for all requests once here.
        let url = Url::from_str(&format!("{base_url}/v{version}/"))
            .context("URL should be created by combining base_url with version through format! macro")?;
        
        Ok(Self {
            client,
            base_url: url,
        })
    }

    /// Sets up a request through [`DiscordHttpClient`]
    /// The [`Method`], [Path][`Url`] and [Headers][`HeaderMap`] are retrieved through a [`DiscordHttpRequest`]
    pub fn request(&self, request: DiscordHttpRequest) ->  Result<RequestBuilder> {

        // Constructs the full URL utilised for this request
        let url_address = self.base_url.join(request.get_request_path())
            .context("Failed to join base_url with Request path")?;

        // Create the Request and send it retrieving the result of the request.
        Ok(self.client.request(request.method, url_address)
            .headers(request.headers))
    }

    /// Method which acts as the processor for recieving inbound a [`DiscordHttpRequest`] to the [`DiscordHttpClient`] channel.
    /// The inbound channel is [multi-producer single-consumer][`MspcReceiver`]
    /// The response channel is [oneshot][`OneshotSender`]
    pub async fn handle_channel_inbound_requests(self, mut reciever: MpscReceiver<DiscordHttpClientRequest>) -> Result<()> {

        // Recieve new inbound requests from the reciever
        while let Some(request) = reciever.recv().await {

            // Create our request to send, 
            // If error is encountered in process of creating the request we send it back & skip to next request in the channel
            // Errors are ignored as the original sender may not wish for a response and this is acceptable behaviour
            let request_builder = match self.request(request.request).context("Failed to create Request Builder") {
                Ok(request) => request,
                Err(error) => {let _ = request.response_sender.send(Err(error)); continue},
            };
            
            

            // Spawn a new task so we can handle sending the request asynchronously so it doesnt block this channel.
            tokio::spawn(
                async move {
                    
                    // Send the request to discord, if an error is encountered send it back through the recieving channel
                    // Errors are ignored as the original sender may not wish for a response and this is acceptable behaviour
                    let response = match request_builder.send().await.context("Failed to send request to Discord Api") {
                        Ok(response) => response,
                        Err(error) => {let _ = request.response_sender.send(Err(error)); return},
                    };

                    

                    // Get the StatusCode of the request
                    let status = response.status();

                    // Check for any issues which can be identified through the status code 
                    let response = match status {
                        reqwest::StatusCode::UNAUTHORIZED => {

                            // Unwrap because we've already confirmed its an Unauthorized error.
                            let error = response.error_for_status().unwrap_err();
                            
                            // Send the error back
                            // If the sender of the original requests no longer wants to recieve a response which is possible behaviour then we can just ignore the error.
                            let _ = request.response_sender.send(Err(anyhow::Error::new(error).context(
                                "Unauthorized most likely due to Invalid Token Passed"
                            )));
                            return

                        },
                        _ => {
                            response
                        },
                    };

                    // Send back the result
                    // Errors are ignored as dropping the reciever is acceptable when it no longer wants the response.
                    let _ = request.response_sender.send(Ok(response));
                }
            );

        }

        Ok(())

    }
}

/// Simplifies sending process for sending a [`DiscordHttpRequest`] through the [`DiscordHttpClient`] mspc request processing channel.
pub async fn send_discord_http_request(request: DiscordHttpRequest, http_client_sender: DiscordHttpClientReqSender) -> Result<HttpResponse> {

    // Create the channel to recieve the response from the DiscordHttpClient channel.
    let (response_sender, response_reciever) = oneshot::channel();

    // Create the Request struct to send through the channel
    let client_request = DiscordHttpClientRequest { 
        response_sender: response_sender, 
        request: request
    };

    // Send it through the request processing channel, expect here because this is a critical error which would cause failure of the bot if it did not panic as no further http requests could be sent.
    http_client_sender.send(client_request)
        .await
        .expect("Attempted to send Request through DiscordHttpClient mpsc channel");

    // Recieve the response from the Request processor and return it, expect here because the sending half should never be disconnected while this hasnt been recieved yet.
    response_reciever.await
        .expect("Attempted to recieve response from DiscordHttpClient channel processor")
    
}