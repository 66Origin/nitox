use futures::{
    future::{self, Either},
    prelude::*,
};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio_executor;

use error::NatsError;
use protocol::Op;

use super::connection_inner::NatsConnectionInner;

macro_rules! reco {
    ($conn:ident) => {
        if let Ok(mut state) = $conn.state.write() {
            *state = NatsConnectionState::Disconnected;
        }

        tokio_executor::spawn($conn.reconnect().map_err(|e| {
            debug!(target: "nitox", "Reconnection error: {}", e);
            ()
        }));
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NatsConnectionState {
    Connected,
    Reconnecting,
    Disconnected,
}

#[derive(Debug)]
pub struct NatsConnection {
    pub(crate) is_tls: bool,
    pub(crate) addr: SocketAddr,
    pub(crate) host: Option<String>,
    pub(crate) inner: Arc<RwLock<NatsConnectionInner>>,
    pub(crate) state: Arc<RwLock<NatsConnectionState>>,
}

impl NatsConnection {
    fn reconnect(&self) -> impl Future<Item = (), Error = NatsError> {
        if let Ok(mut state) = self.state.write() {
            *state = NatsConnectionState::Reconnecting;
        }

        let inner_arc = Arc::clone(&self.inner);
        let inner_state = Arc::clone(&self.state);
        let is_tls = self.is_tls;
        let maybe_host = self.host.clone();
        NatsConnectionInner::connect_tcp(&self.addr)
            .and_then(move |socket| {
                if is_tls {
                    Either::A(
                        NatsConnectionInner::upgrade_tcp_to_tls(&maybe_host.unwrap(), socket)
                            .map(|socket| NatsConnectionInner::from(socket)),
                    )
                } else {
                    Either::B(future::ok(NatsConnectionInner::from(socket)))
                }
            }).and_then(move |inner| {
                let res = if let Ok(mut inner_conn) = inner_arc.write() {
                    *inner_conn = inner;
                    if let Ok(mut state) = inner_state.write() {
                        *state = NatsConnectionState::Connected;
                    }
                    debug!(target: "nitox", "Successfully swapped reconnected underlying connection");
                    Ok(())
                } else {
                    debug!(target: "nitox", "Cannot reconnect to server");
                    Err(NatsError::CannotReconnectToServer)
                };

                res
            })
    }
}

impl Sink for NatsConnection {
    type SinkError = NatsError;
    type SinkItem = Op;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        if let Ok(mut inner) = self.inner.try_write() {
            match inner.start_send(item.clone()) {
                Err(NatsError::ServerDisconnected(_)) => {
                    reco!(self);
                    Ok(AsyncSink::NotReady(item))
                }
                poll_res => poll_res,
            }
        } else {
            Ok(AsyncSink::NotReady(item))
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        if let Ok(mut inner) = self.inner.try_write() {
            match inner.poll_complete() {
                Err(NatsError::ServerDisconnected(_)) => {
                    reco!(self);
                    Ok(Async::NotReady)
                }
                poll_res => poll_res,
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}

impl Stream for NatsConnection {
    type Error = NatsError;
    type Item = Op;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        if let Ok(mut inner) = self.inner.try_write() {
            match inner.poll() {
                Err(NatsError::ServerDisconnected(_)) => {
                    reco!(self);
                    Ok(Async::NotReady)
                }
                poll_res => poll_res,
            }
        } else {
            Ok(Async::NotReady)
        }
    }
}
