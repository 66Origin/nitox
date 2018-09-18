use error::NatsError;
use futures::{
    future::{self, Either},
    prelude::*,
    Future,
};
use net::{
    connect::*,
    reconnect::{Reconnect, ReconnectError},
};
use protocol::{commands::*, CommandError, Op};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone)]
struct NatsClientInner {
    connection: Arc<NatsConnection>,
}
// TODO: try writing abstraction to solve borrowing problem? I have no clue. Sleep time now

#[derive(Debug, Default, Clone, Builder)]
pub struct NatsClientOptions {
    connect_command: ConnectCommand,
    cluster_uri: String,
}

#[derive(Debug)]
pub struct NatsClient {
    opts: NatsClientOptions,
    inner: NatsClientInner,
}

impl NatsClient {
    pub fn from_options(opts: NatsClientOptions) -> impl Future<Item = Self, Error = NatsError> {
        let cluster_uri = opts.cluster_uri.clone();
        let tls_required = opts.connect_command.tls_required.clone();

        future::result(SocketAddr::from_str(&cluster_uri))
            .from_err()
            .and_then(move |cluster_sa| {
                if tls_required {
                    match Url::parse(&cluster_uri) {
                        Ok(url) => match url.host_str() {
                            Some(host) => future::ok(Either::B(connect_tls(host.to_string(), &cluster_sa))),
                            None => future::err(NatsError::TlsHostMissingError),
                        },
                        Err(e) => future::err(e.into()),
                    }
                } else {
                    future::ok(Either::A(connect(&cluster_sa)))
                }
            }).and_then(|either| either)
            .map(move |connection| NatsClient {
                inner: NatsClientInner {
                    connection: Arc::new(connection),
                },
                opts,
            })
    }

    /*pub fn publish(&self, cmd: PubCommand) -> impl Future<Item = NatsConnection, Error = NatsError> {
        //let maybe_inbox = cmd.reply_to.clone();
        //let conn = self.connection.borrow();
        self.inner.send(Op::PUB(cmd)).into_future() /*.and_then(move |conn| {
            if let Some(inbox) = maybe_inbox {
                Either::A(
                    conn.skip_while(move |op: &Op| {
                        if let Op::MSG(msg) = op {
                            if let Some(ref msg_inbox) = msg.reply_to {
                                return Ok(msg_inbox != &inbox);
                            }
                        }

                        Ok(true)
                    }).take(1)
                    .into_future()
                    .map_err(|(e, _)| e)
                    .and_then(|(op, _): (Option<Op>, _)| {
                        if let Some(Op::MSG(msg)) = op {
                            future::ok(Some(msg.clone()))
                        } else {
                            future::ok(None)
                        }
                    }),
                )
            } else {
                Either::B(future::ok(None))
            }
        })*/
    }*/

    /*pub fn subscribe(&self, cmd: SubCommand) -> impl Future<Item = SubCommand, Error = NatsError> {
        conn.send(Op::SUB(cmd.clone())).into_future().map(|_| cmd)
    }

    pub fn unsubscribe(&self, cmd: UnsubCommand) -> impl Future<Item = UnsubCommand, Error = NatsError> {
        conn.send(Op::UNSUB(cmd.clone())).into_future().map(|_| cmd)
    }*/
}
