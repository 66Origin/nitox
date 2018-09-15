use super::client::*;
use super::server::*;
use super::{Command, CommandError};
use bytes::{BufMut, Bytes, BytesMut};

/// Abstraction over NATS protocol messages
pub enum Op {
    /// [SERVER] Sent to client after initial TCP/IP connection
    INFO(ServerInfo),
    /// [CLIENT] Sent to server to specify connection information
    CONNECT(Connect),
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

impl Op {
    pub fn to_bytes(self) -> Result<Bytes, CommandError> {
        Ok(match self {
            Op::INFO(si) => si.into_vec()?,
            Op::CONNECT(con) => con.into_vec()?,
            Op::PUB(pc) => pc.into_vec()?,
            Op::SUB(sc) => sc.into_vec()?,
            Op::UNSUB(uc) => uc.into_vec()?,
            Op::MSG(msg) => msg.into_vec()?,
            Op::PING => format!("PING\r\n").as_bytes().into(),
            Op::PONG => format!("PONG\r\n").as_bytes().into(),
            Op::OK => format!("+OK\r\n").as_bytes().into(),
            Op::ERR(se) => format!("-ERR {}\r\n", se).as_bytes().into(),
        })
    }

    pub fn from_bytes(buf: &mut BytesMut) -> Result<Self, CommandError> {
        // TODO: Decode stuff :O
        unimplemented!()
    }
}
