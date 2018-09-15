use protocol::{Command, CommandError};
use serde_json as json;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct Connect {
    /// Turns on +OK protocol acknowledgements.
    verbose: bool,
    /// Turns on additional strict format checking, e.g. for properly formed subjects
    pedantic: bool,
    /// Indicates whether the client requires an SSL connection.
    tls_required: bool,
    /// Client authorization token (if auth_required is set)
    auth_token: Option<String>,
    /// Connection username (if auth_required is set)
    user: Option<String>,
    /// Connection password (if auth_required is set)
    pass: Option<String>,
    /// Optional client name
    name: Option<String>,
    /// The implementation language of the client.
    lang: String,
    /// The version of the client.
    version: String,
    /// optional int. Sending 0 (or absent) indicates client supports original protocol. Sending 1 indicates that the client supports dynamic reconfiguration of cluster topology changes by asynchronously receiving INFO messages with known servers it can reconnect to.
    protocol: Option<u8>,
    /// Optional boolean. If set to true, the server (version 1.2.0+) will not send originating messages from this connection to its own subscriptions. Clients should set this to true only for server supporting this feature, which is when proto in the INFO protocol is set to at least 1.
    echo: Option<bool>,
}

impl Command for Connect {
    fn into_vec(self) -> Result<Vec<u8>, CommandError> {
        Ok(format!("CONNECT\t{}\r\n", json::to_string(&self)?)
            .as_bytes()
            .to_vec())
    }
}
