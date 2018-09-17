use bytes::Bytes;
use protocol::{Command, CommandError};
use serde_json as json;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Builder)]
pub struct ConnectCommand {
    /// Turns on +OK protocol acknowledgements.
    pub verbose: bool,
    /// Turns on additional strict format checking, e.g. for properly formed subjects
    pub pedantic: bool,
    /// Indicates whether the client requires an SSL connection.
    pub tls_required: bool,
    /// Client authorization token (if auth_required is set)
    auth_token: Option<String>,
    /// Connection username (if auth_required is set)
    user: Option<String>,
    /// Connection password (if auth_required is set)
    pass: Option<String>,
    /// Optional client name
    pub name: Option<String>,
    /// The implementation language of the client.
    #[builder(default = "\"rust\"")]
    pub lang: &'static str,
    /// The version of the client.
    #[builder(default = "self.default_ver()?")]
    pub version: String,
    /// optional int. Sending 0 (or absent) indicates client supports original protocol. Sending 1 indicates that the
    /// client supports dynamic reconfiguration of cluster topology changes by asynchronously receiving INFO messages
    /// with known servers it can reconnect to.
    protocol: Option<u8>,
    /// Optional boolean. If set to true, the server (version 1.2.0+) will not send originating messages from this
    /// connection to its own subscriptions. Clients should set this to true only for server supporting this feature,
    /// which is when proto in the INFO protocol is set to at least 1.
    echo: Option<bool>,
}
impl ConnectCommandBuilder {
    fn default_ver(&self) -> Result<String, String> {
        match ::std::env::var("CARGO_PKG_VERSION") {
            Ok(v) => Ok(v),
            Err(_) => Err("Package version not found in env".into()),
        }
    }
}

impl Command for ConnectCommand {
    const CMD_NAME: &'static [u8] = b"CONNNECT";

    fn into_vec(self) -> Result<Bytes, CommandError> {
        Ok(format!("CONNECT\t{}\r\n", json::to_string(&self)?).as_bytes().into())
    }

    fn try_parse(buf: &[u8]) -> Result<Self, CommandError> {
        let len = buf.len();

        if buf[len - 2..] != [b'\r', b'\n'] {
            return Err(CommandError::IncompleteCommandError);
        }
        // Check if we're still on the right command
        if buf[..7] != *Self::CMD_NAME {
            return Err(CommandError::CommandMalformed);
        }

        Ok(json::from_slice(&buf[7..len - 2].to_vec())?)
    }
}
