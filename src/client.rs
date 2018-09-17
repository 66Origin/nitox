use error::NatsError;
use futures::{
    future::{err as FutErr, ok as FutOk, Either},
    Future,
};
use net::{
    connect::*,
    reconnect::{Reconnect, ReconnectError},
};
use protocol::{commands::*, CommandError};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio_io::{AsyncRead, AsyncWrite};
use url::{Host, Url};

type InnerNatsConnection = Either<
    Box<dyn Future<Item = NatsConnection, Error = NatsError>>,
    Box<dyn Future<Item = TLSNatsConnection, Error = NatsError>>,
>;

#[derive(Debug, Default, Clone, Builder)]
pub struct NatsClientOptions {
    connect_command: ConnectCommand,
    cluster_uri: String,
}

#[derive(Debug)]
pub struct NatsClient<T: AsyncRead + AsyncWrite> {
    connection: T,
}

impl<T: AsyncRead + AsyncWrite> NatsClient<T> {
    pub fn from_options(opts: NatsClientOptions) -> impl Future<Item = Self, Error = NatsError> {
        let NatsClientOptions {
            cluster_uri,
            connect_command,
        } = opts;

        let cluster_sa = match SocketAddr::from_str(&cluster_uri) {
            Ok(sa) => sa,
            Err(e) => return FutErr(e.into()),
        };

        let nats_conn: InnerNatsConnection = if connect_command.tls_required {
            match Url::parse(&cluster_uri) {
                Ok(url) => match url.host_str() {
                    Some(host) => Either::B(Box::new(connect_tls(host.to_string(), &cluster_sa))),
                    None => return FutErr(NatsError::GenericError("Host is missing".into())),
                },
                Err(e) => return FutErr(e.into()),
            }
        } else {
            Either::A(Box::new(connect(&cluster_sa)))
        };

        // FIXME: NANI
        nats_conn.and_then(move |connection| NatsClient { connection })
    }
}
