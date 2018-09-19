use bytes::Bytes;
use protocol::{Command, CommandError};
use serde_json as json;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ServerInfo {
    /// The unique identifier of the NATS server
    server_id: String,
    /// The version of the NATS server
    version: String,
    /// The version of golang the NATS server was built with
    proto: u8,
    /// The IP address used to start the NATS server, by default this will be 0.0.0.0 and can be configured with
    /// `-client_advertise host:port`
    go: String,
    /// The port number the NATS server is configured to listen on
    host: String,
    /// Maximum payload size, in bytes, that the server will accept from the client.
    port: u32,
    /// An integer indicating the protocol version of the server. The server version 1.2.0 sets this to 1 to indicate
    /// that it supports the “Echo” feature.
    max_payload: u32,
    /// An optional unsigned integer (64 bits) representing the internal client identifier in the server. This can be
    /// used to filter client connections in monitoring, correlate with error logs, etc…
    #[builder(default)]
    client_id: Option<u64>,
    /// If this is set, then the client should try to authenticate upon connect.
    #[builder(default)]
    auth_required: Option<bool>,
    /// If this is set, then the client must perform the TLS/1.2 handshake. Note, this used to be ssl_required and has
    /// been updated along with the protocol from SSL to TLS.
    #[builder(default)]
    tls_required: Option<bool>,
    /// If this is set, the client must provide a valid certificate during the TLS handshake.
    #[builder(default)]
    tls_verify: Option<bool>,
    /// An optional list of server urls that a client can connect to.
    #[builder(default)]
    connect_urls: Option<Vec<String>>,
}

impl Command for ServerInfo {
    const CMD_NAME: &'static [u8] = b"INFO";

    fn into_vec(self) -> Result<Bytes, CommandError> {
        Ok(format!("INFO\t{}\r\n", json::to_string(&self)?).as_bytes().into())
    }

    fn try_parse(buf: &[u8]) -> Result<Self, CommandError> {
        let len = buf.len();

        if buf[len - 2..] != [b'\r', b'\n'] {
            return Err(CommandError::IncompleteCommandError);
        }
        // Check if we're still on the right command
        if buf[..4] != *Self::CMD_NAME {
            return Err(CommandError::CommandMalformed);
        }

        Ok(json::from_slice(&buf[4..len - 2])?)
    }
}
