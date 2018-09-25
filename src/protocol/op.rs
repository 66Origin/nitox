use super::{commands::*, Command, CommandError};
use bytes::Bytes;

/// Abstraction over NATS protocol messages
#[derive(Debug, Clone)]
pub enum Op {
    /// [SERVER] Sent to client after initial TCP/IP connection
    INFO(ServerInfo),
    /// [CLIENT] Sent to server to specify connection information
    CONNECT(ConnectCommand),
    /// [CLIENT] Publish a message to a subject, with optional reply subject
    PUB(PubCommand),
    /// [CLIENT] Subscribe to a subject (or subject wildcard)
    SUB(SubCommand),
    /// [CLIENT] Unsubscribe (or auto-unsubscribe) from subject
    UNSUB(UnsubCommand),
    /// [SERVER] Delivers a message payload to a subscriber
    MSG(Message),
    /// [BOTH] PING keep-alive message
    PING,
    /// [BOTH] PONG keep-alive message
    PONG,
    /// [SERVER] Acknowledges well-formed protocol message in `verbose` mode
    OK,
    /// [SERVER] Indicates a protocol error. May cause client disconnect.
    ERR(ServerError),
}

macro_rules! op_from_cmd {
    ($buf:ident, $cmd:path, $op:path) => {{
        use protocol::CommandError;

        match $cmd(&$buf) {
            Ok(c) => Ok(Some($op(c))),
            Err(CommandError::IncompleteCommandError) => return Err(CommandError::IncompleteCommandError),
            Err(e) => return Err(e.into()),
        }
    }};
}

impl Op {
    pub fn into_bytes(self) -> Result<Bytes, CommandError> {
        Ok(match self {
            Op::INFO(si) => si.into_vec()?,
            Op::CONNECT(con) => con.into_vec()?,
            Op::PUB(pc) => pc.into_vec()?,
            Op::SUB(sc) => sc.into_vec()?,
            Op::UNSUB(uc) => uc.into_vec()?,
            Op::MSG(msg) => msg.into_vec()?,
            Op::PING => "PING\r\n".into(),
            Op::PONG => "PONG\r\n".into(),
            Op::OK => "+OK\r\n".into(),
            Op::ERR(se) => format!("-ERR {}\r\n", se).as_bytes().into(),
        })
    }

    pub fn from_bytes(cmd_name: &[u8], buf: &[u8]) -> Result<Option<Self>, CommandError> {
        match cmd_name {
            ServerInfo::CMD_NAME => op_from_cmd!(buf, ServerInfo::try_parse, Op::INFO),
            ConnectCommand::CMD_NAME => op_from_cmd!(buf, ConnectCommand::try_parse, Op::CONNECT),
            Message::CMD_NAME => op_from_cmd!(buf, Message::try_parse, Op::MSG),
            PubCommand::CMD_NAME => op_from_cmd!(buf, PubCommand::try_parse, Op::PUB),
            SubCommand::CMD_NAME => op_from_cmd!(buf, SubCommand::try_parse, Op::SUB),
            UnsubCommand::CMD_NAME => op_from_cmd!(buf, UnsubCommand::try_parse, Op::UNSUB),
            b"+OK" => Ok(Some(Op::OK)),
            b"-ERR" => Ok(Some(Op::ERR(ServerError::from(String::from_utf8(buf[1..].to_vec())?)))),
            _ => {
                if buf.len() > 7 {
                    Err(CommandError::CommandNotFoundOrSupported)
                } else {
                    Ok(None)
                }
            },
        }
    }
}
