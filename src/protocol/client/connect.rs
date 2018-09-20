use bytes::Bytes;
use protocol::{Command, CommandError};
use serde_json as json;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize, Builder)]
#[builder(default)]
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
    #[builder(default = "self.default_lang()?")]
    pub lang: String,
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

    fn default_lang(&self) -> Result<String, String> {
        Ok(String::from("rust"))
    }
}

impl Command for ConnectCommand {
    const CMD_NAME: &'static [u8] = b"CONNNECT";

    fn into_vec(self) -> Result<Bytes, CommandError> {
        Ok(format!("CONNECT\t{}\r\n", json::to_string(&self)?).as_bytes().into())
    }

    fn try_parse(buf: &[u8]) -> Result<ConnectCommand, CommandError> {
        let len = buf.len();

        if buf[len - 2..] != [b'\r', b'\n'] {
            return Err(CommandError::IncompleteCommandError);
        }
        // Check if we're still on the right command
        if buf[..7] != *Self::CMD_NAME {
            return Err(CommandError::CommandMalformed);
        }

        Ok(json::from_slice(&buf[7..len - 2])?)
    }
}

#[cfg(test)]
mod connect_command_tests {
    use super::{ConnectCommand, ConnectCommandBuilder};
    use protocol::Command;

    static DEFAULT_CONNECT: &'static [u8] = b"CONNECT {\"verbose\":false,\"pedantic\":false,\"tls_required\":false,\"name\":\"nitox\",\"lang\":\"rust\",\"version\":\"1.0.0\",\"protocol\":1}\r\n";

    #[test]
    fn it_parses() {
        let parse_res = ConnectCommand::try_parse(DEFAULT_CONNECT);
        assert!(parse_res.is_ok());
    }

    #[test]
    fn it_stringifies() {
        let cmd = ConnectCommandBuilder::default()
            .lang("rust".into())
            .version("1.0.0".into())
            .name(Some("nitox".into()))
            .build()
            .unwrap();

        let cmd_bytes_res = cmd.into_vec();
        assert!(cmd_bytes_res.is_ok());
        let cmd_bytes = cmd_bytes_res.unwrap();

        assert_eq!(DEFAULT_CONNECT, cmd_bytes);
    }
}
