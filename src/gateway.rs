use tokio::net::*;
use tokio_tungstenite::*;

/// A type alias which exists to shorten the main websocket stream.
pub type WebsocketTCPStream = WebSocketStream<MaybeTlsStream<TcpStream>>;


/// A connection to the Discord Gateway.
pub struct Gateway { 

    /// The main I/O stream for the gateway.
    pub gateway_stream: WebsocketTCPStream,

    /// The id for the shard which this [`Gateway`] represents.
    pub shard_id: u32,

    /// The total amount of shards present.
    pub shard_total: u32,
}

