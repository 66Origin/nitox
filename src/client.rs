use protocol::{commands::*, CommandError};
use std::net::SocketAddr;

#[derive(Debug, Clone, Builder)]
pub struct NatsClientOptions {
    connect_command: ConnectCommand,
    cluster_addr: SocketAddr,
}

impl NatsClientOptions {}

#[derive(Debug, Clone, Builder)]
pub struct NatsClient {
    // TODO:
}

impl NatsClient {
    pub fn new() -> Self {
        NatsClient {}
    }
}
