use bytes::Bytes;
use error::NatsError;
use futures::{
    future::{self, Either},
    prelude::*,
    stream,
    sync::mpsc,
    Future,
};
use net::{
    connect::*,
    reconnect::{Reconnect, ReconnectError},
};
use protocol::{commands::*, CommandError, Op};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio_executor;
use url::Url;

type NatsSink = stream::SplitSink<NatsConnection>;
type NatsStream = stream::SplitStream<NatsConnection>;
type NatsSubscriptionId = String;

#[derive(Clone)]
struct NatsClientSender {
    tx: mpsc::UnboundedSender<Op>,
    verbose: bool,
}

impl NatsClientSender {
    pub fn new(conn: NatsConnection) -> (Self, NatsStream) {
        let (sink, stream): (NatsSink, NatsStream) = conn.split();
        let (tx, rx) = mpsc::unbounded();
        let rx = rx.map_err(|_| NatsError::InnerBrokenChain);
        let work = sink.send_all(rx).map(|_| ()).map_err(|_| ());
        tokio_executor::spawn(work);

        (NatsClientSender { tx, verbose: false }, stream)
    }

    #[allow(dead_code)]
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn send(&self, op: Op) -> impl Future<Item = (), Error = NatsError> {
        let _verbose = self.verbose.clone();
        let fut = self
            .tx
            .unbounded_send(op)
            .map_err(|_| NatsError::InnerBrokenChain)
            .into_future();

        fut
    }
}

#[derive(Debug)]
struct NatsClientMultiplexer {
    subs_tx: Arc<RwLock<HashMap<NatsSubscriptionId, mpsc::UnboundedSender<Op>>>>,
}

impl NatsClientMultiplexer {
    pub fn new(stream: stream::SplitStream<NatsConnection>) -> Self {
        let (tx, inner_rx) = mpsc::unbounded();

        // Forward the whole TCP incoming stream to the above channel
        let work_tx = stream.forward(tx).map(|_| ()).map_err(|_| ());
        tokio_executor::spawn(work_tx);

        let subs_tx: Arc<RwLock<HashMap<NatsSubscriptionId, mpsc::UnboundedSender<Op>>>> =
            Arc::new(RwLock::new(HashMap::default()));

        let stx_inner = subs_tx.clone();
        // Here we filter the incoming TCP stream Messages by subscription ID and sending it to the appropriate Sender
        let work_rx = inner_rx
            .and_then(move |op| {
                match &op {
                    Op::MSG(msg) => {
                        if let Ok(stx) = stx_inner.read() {
                            if let Some(tx) = stx.get(&msg.sid) {
                                let _ = tx.unbounded_send(op.clone());
                            }
                        }
                    }
                    // Take care of the rest of the messages? Like ping and so on
                    _ => {}
                }

                future::ok::<(), ()>(())
            }).into_future()
            .map(|_| ())
            .map_err(|_| ());

        tokio_executor::spawn(work_rx);

        NatsClientMultiplexer { subs_tx }
    }

    pub fn for_sid(&self, sid: NatsSubscriptionId) -> impl Stream<Item = Message, Error = NatsError> {
        let (tx, rx) = mpsc::unbounded();
        if let Ok(mut subs) = self.subs_tx.write() {
            subs.insert(sid.clone(), tx);
        }

        rx.filter_map(move |op| {
            if let Op::MSG(msg) = op {
                if msg.sid == sid {
                    return Some(msg);
                }
            }

            None
        }).map_err(|_| NatsError::InnerBrokenChain)
    }
}

#[derive(Debug, Default, Clone, Builder)]
pub struct NatsClientOptions {
    connect_command: ConnectCommand,
    cluster_uri: String,
}

pub struct NatsClient {
    opts: NatsClientOptions,
    tx: NatsClientSender,
    rx: Arc<NatsClientMultiplexer>,
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
            .map(move |connection| {
                let (tx, stream) = NatsClientSender::new(connection);
                let rx = NatsClientMultiplexer::new(stream);
                NatsClient {
                    tx,
                    rx: Arc::new(rx),
                    opts,
                }
            })
    }

    pub fn connect(self) -> impl Future<Item = Self, Error = NatsError> {
        self.tx
            .send(Op::CONNECT(self.opts.connect_command.clone()))
            .into_future()
            .and_then(move |_| future::ok(self))
    }

    pub fn publish(&self, cmd: PubCommand) -> impl Future<Item = (), Error = NatsError> {
        self.tx.send(Op::PUB(cmd)).map(|r| r).into_future()
    }

    pub fn unsubscribe(&self, cmd: UnsubCommand) -> impl Future<Item = (), Error = NatsError> {
        self.tx.send(Op::UNSUB(cmd)).map(|r| r).into_future()
    }

    pub fn subscribe(&self, cmd: SubCommand) -> impl Stream<Item = Message, Error = NatsError> {
        self.tx.send(Op::SUB(cmd.clone()));
        self.rx.for_sid(cmd.sid)
    }

    pub fn request(&self, subject: String, payload: Bytes) -> impl Future<Item = Message, Error = NatsError> {
        let inbox = PubCommandBuilder::generate_reply_to();
        let pub_cmd = PubCommand {
            subject,
            payload,
            reply_to: Some(inbox.clone()),
        };

        let sub_cmd = SubCommand {
            queue_group: None,
            sid: SubCommandBuilder::generate_sid(),
            subject: inbox,
        };

        let sid = sub_cmd.sid.clone();

        let unsub_cmd = UnsubCommand {
            sid: sub_cmd.sid.clone(),
            max_msgs: Some(1),
        };

        let tx1 = self.tx.clone();
        let tx2 = self.tx.clone();
        let rx = Arc::clone(&self.rx);
        self.tx
            .send(Op::SUB(sub_cmd))
            .and_then(move |_| tx1.send(Op::UNSUB(unsub_cmd)))
            .and_then(move |_| tx2.send(Op::PUB(pub_cmd)))
            .and_then(move |_| {
                rx.for_sid(sid)
                    .into_future()
                    .map(|(maybe_message, _)| maybe_message.unwrap())
                    .map_err(|_| NatsError::InnerBrokenChain)
            })
    }
}
