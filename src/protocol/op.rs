use serde_json as json;
use std::fmt;

use super::server::*;

pub enum Op {
    /// [SERVER] Sent to client after initial TCP/IP connection
    INFO(ServerInfo),
    /// [CLIENT] Sent to server to specify connection information
    CONNECT(Connect),
    /// [CLIENT] Publish a message to a subject, with optional reply subject
    PUB,
    /// [CLIENT] Subscribe to a subject (or subject wildcard)
    SUB,
    /// [CLIENT] Unsubscribe (or auto-unsubscribe) from subject
    UNSUB,
    /// [SERVER] Delivers a message payload to a subscriber
    MSG,
    /// [BOTH] PING keep-alive message
    PING,
    /// [BOTH] PONG keep-alive message
    PONG,
    /// [SERVER] Acknowledges well-formed protocol message in `verbose` mode
    OK,
    /// [SERVER] Indicates a protocol error. May cause client disconnect.
    ERR,
}

impl Op {
    fn format(&self) -> Result<String, json::Error> {
        Ok(match self {
            Op::INFO(si) => format!("INFO {}", json::to_string(si)?),
            Op::CONNECT(con) => format!("CONNECT {}", json::to_string(con)?),
            Op::PUB => format!("PUB"),
            Op::SUB => format!("SUB"),
            Op::UNSUB => format!("UNSUB"),
            Op::MSG => format!("MSG"),
            Op::PING => format!("PING"),
            Op::PONG => format!("PONG"),
            Op::OK => format!("+OK"),
            Op::ERR => format!("-ERR"),
        })
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.format() {
            Ok(s) => write!(f, "{}", s),
            Err(_) => Err(fmt::Error),
        }
    }
}
