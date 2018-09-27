use bytes::Bytes;

use futures::{
    future::{self, Either},
    prelude::*,
    stream,
    sync::mpsc,
    Future,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, RwLock},
};
use tokio_executor;
use url::Url;

use error::NatsError;
use net::*;
use protocol::{commands::*, Op};

type NatsSink = stream::SplitSink<NatsConnection>;
type NatsStream = stream::SplitStream<NatsConnection>;
type NatsSubscriptionId = String;

#[derive(Clone, Debug)]
struct NatsClientSender {
    tx: mpsc::UnboundedSender<Op>,
    verbose: bool,
}

impl NatsClientSender {
    pub fn new(sink: NatsSink) -> Self {
        let (tx, rx) = mpsc::unbounded();
        let rx = rx.map_err(|_| NatsError::InnerBrokenChain);
        let work = sink.send_all(rx).map(|_| ()).map_err(|_| ());
        tokio_executor::spawn(work);

        NatsClientSender { tx, verbose: false }
    }

    #[allow(dead_code)]
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    pub fn send(&self, op: Op) -> impl Future<Item = (), Error = NatsError> {
        //let _verbose = self.verbose.clone();
        self.tx
            .unbounded_send(op)
            .map_err(|_| NatsError::InnerBrokenChain)
            .into_future()
    }
}

#[derive(Debug)]
struct NatsClientMultiplexer {
    other_tx: Arc<mpsc::UnboundedSender<Op>>,
    subs_tx: Arc<RwLock<HashMap<NatsSubscriptionId, mpsc::UnboundedSender<Message>>>>,
}

impl NatsClientMultiplexer {
    pub fn new(stream: NatsStream) -> (Self, mpsc::UnboundedReceiver<Op>) {
        let subs_tx: Arc<RwLock<HashMap<NatsSubscriptionId, mpsc::UnboundedSender<Message>>>> =
            Arc::new(RwLock::new(HashMap::default()));

        let (other_tx, other_rx) = mpsc::unbounded();
        let other_tx = Arc::new(other_tx);

        let stx_inner = Arc::clone(&subs_tx);
        let otx_inner = Arc::clone(&other_tx);

        // Here we filter the incoming TCP stream Messages by subscription ID and sending it to the appropriate Sender
        let work_tx = stream
            .for_each(move |op| {
                match op {
                    Op::MSG(msg) => {
                        if let Ok(stx) = stx_inner.read() {
                            if let Some(tx) = stx.get(&msg.sid) {
                                let _ = tx.unbounded_send(msg);
                            }
                        }
                    }
                    // Forward the rest of the messages to the owning client
                    op => {
                        let _ = otx_inner.unbounded_send(op);
                    }
                }

                future::ok::<(), NatsError>(())
            }).map(|_| ())
            .map_err(|_| ());

        tokio_executor::spawn(work_tx);

        (NatsClientMultiplexer { subs_tx, other_tx }, other_rx)
    }

    pub fn for_sid(&self, sid: NatsSubscriptionId) -> impl Stream<Item = Message, Error = NatsError> {
        let (tx, rx) = mpsc::unbounded();
        if let Ok(mut subs) = self.subs_tx.write() {
            subs.insert(sid, tx);
        }

        rx.map_err(|_| NatsError::InnerBrokenChain)
    }

    pub fn remove_sid(&self, sid: &str) {
        if let Ok(mut subs) = self.subs_tx.write() {
            subs.remove(sid);
        }
    }
}

#[derive(Debug, Default, Clone, Builder)]
#[builder(setter(into))]
pub struct NatsClientOptions {
    connect_command: ConnectCommand,
    cluster_uri: String,
}

pub struct NatsClient {
    opts: NatsClientOptions,
    other_rx: Box<dyn Stream<Item = Op, Error = NatsError> + Send>,
    tx: NatsClientSender,
    rx: Arc<NatsClientMultiplexer>,
}

impl ::std::fmt::Debug for NatsClient {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            f,
            "NatsClient {{ opts: {:?}, other_rx: [REDACTED], tx: {:?}, rx: {:?} }}",
            self.opts, self.tx, self.rx
        )
    }
}

impl Stream for NatsClient {
    type Error = NatsError;
    type Item = Op;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        self.other_rx.poll().map_err(|_| NatsError::InnerBrokenChain)
    }
}

impl NatsClient {
    pub fn from_options(opts: NatsClientOptions) -> impl Future<Item = Self, Error = NatsError> {
        let cluster_uri = opts.cluster_uri.clone();
        let tls_required = opts.connect_command.tls_required;

        future::result(SocketAddr::from_str(&cluster_uri))
            .from_err()
            .and_then(move |cluster_sa| {
                if tls_required {
                    match Url::parse(&cluster_uri) {
                        Ok(url) => match url.host_str() {
                            Some(host) => future::ok(Either::B(connect_tls(host.to_string(), cluster_sa))),
                            None => future::err(NatsError::TlsHostMissingError),
                        },
                        Err(e) => future::err(e.into()),
                    }
                } else {
                    future::ok(Either::A(connect(cluster_sa)))
                }
            }).and_then(|either| either)
            .and_then(move |connection| {
                let (sink, stream): (NatsSink, NatsStream) = connection.split();
                let (rx, other_rx) = NatsClientMultiplexer::new(stream);
                let tx = NatsClientSender::new(sink);

                let (tmp_other_tx, tmp_other_rx) = mpsc::unbounded();
                let tx_inner = tx.clone();
                tokio_executor::spawn(
                    other_rx
                        .for_each(move |op| {
                            match op {
                                Op::PING => {
                                    tokio_executor::spawn(tx_inner.send(Op::PONG).map_err(|_| ()));
                                }
                                o => {
                                    let _ = tmp_other_tx.unbounded_send(o);
                                }
                            }
                            future::ok(())
                        }).into_future()
                        .map_err(|_| ()),
                );

                let client = NatsClient {
                    tx,
                    other_rx: Box::new(tmp_other_rx.map_err(|_| NatsError::InnerBrokenChain)),
                    rx: Arc::new(rx),
                    opts,
                };

                future::ok(client)
            })
    }

    pub fn connect(self) -> impl Future<Item = Self, Error = NatsError> {
        self.tx
            .send(Op::CONNECT(self.opts.connect_command.clone()))
            .into_future()
            .and_then(move |_| future::ok(self))
    }

    pub fn send(self, op: Op) -> impl Future<Item = Self, Error = NatsError> {
        self.tx.send(op).into_future().and_then(move |_| future::ok(self))
    }

    pub fn publish(&self, cmd: PubCommand) -> impl Future<Item = (), Error = NatsError> {
        self.tx.send(Op::PUB(cmd)).map(|r| r).into_future()
    }

    pub fn unsubscribe(&self, cmd: UnsubCommand) -> impl Future<Item = (), Error = NatsError> {
        self.tx.send(Op::UNSUB(cmd)).map(|r| r).into_future()
    }

    pub fn subscribe(&self, cmd: SubCommand) -> impl Future<Item = impl Stream<Item = Message, Error = NatsError>> {
        let inner_rx = self.rx.clone();
        let sid = cmd.sid.clone();
        self.tx
            .send(Op::SUB(cmd))
            .and_then(move |_| future::ok(inner_rx.for_sid(sid)))
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
                    .inspect(|msg| println!("msg {:#?}", msg))
                    .take(1)
                    .into_future()
                    .map(|(maybe_message, _)| maybe_message.unwrap())
                    .map_err(|_| NatsError::InnerBrokenChain)
            })
    }
}

#[cfg(test)]
mod client_test {
    use super::*;
    extern crate env_logger;
    extern crate tokio;
    use codec::OpCodec;
    use futures::{
        sync::{mpsc, oneshot},
        Future, Stream,
    };
    use protocol::Op;
    use std::sync::RwLock;
    use tokio_codec::Decoder;
    use tokio_tcp::TcpListener;

    macro_rules! elog {
        () => {
            let _ = env_logger::try_init();
        };
    }

    fn create_tcp_mock(
        runtime: &mut tokio::runtime::Runtime,
        port: usize,
        is_verbose: Option<bool>,
    ) -> Result<(), NatsError> {
        let verbose = is_verbose.unwrap_or(false);
        let listener = TcpListener::bind(&format!("127.0.0.1:{}", port).parse()?)?;
        debug!(target: "nitox", "TCP Mock NATS Server started on port {}", port);
        runtime.spawn(
            listener
                .incoming()
                .map(move |socket| OpCodec::default().framed(socket))
                .from_err()
                .and_then(|socket| socket.send(Op::PING))
                .and_then(move |socket| {
                    let (sink, stream) = socket.split();
                    let (tx, rx) = mpsc::unbounded();
                    let rx = rx.map_err(|_| NatsError::InnerBrokenChain);
                    tokio_executor::spawn(sink.send_all(rx).map(|_| ()).map_err(|_| ()));

                    let sid_lock = RwLock::new(String::new());

                    stream.for_each(move |op| {
                        debug!(target: "nitox", "Got OP from client {:#?}", op);
                        match op {
                            Op::PONG => {
                                println!("Got PONG from client");
                                if verbose {
                                    let _ = tx.unbounded_send(Op::OK);
                                }
                            }
                            Op::PING => {
                                if verbose {
                                    let _ = tx.unbounded_send(Op::OK);
                                }
                                let _ = tx.unbounded_send(Op::PONG);
                            }
                            Op::SUB(cmd) => {
                                if verbose {
                                    let _ = tx.unbounded_send(Op::OK);
                                }

                                if let Ok(mut sid) = sid_lock.write() {
                                    *sid = cmd.sid;
                                }
                            }
                            Op::PUB(cmd) => {
                                if verbose {
                                    let _ = tx.unbounded_send(Op::OK);
                                }
                                let mut builder = MessageBuilder::default();
                                let sub = cmd.subject.clone();
                                builder.subject(cmd.reply_to.unwrap_or(sub));
                                if let Ok(sid) = sid_lock.read() {
                                    builder.sid((*sid).clone());
                                }
                                builder.payload("bar");

                                let msg = builder.build().unwrap();
                                let _ = tx.unbounded_send(Op::MSG(msg));
                            }
                            _ => {
                                if verbose {
                                    let _ = tx.unbounded_send(Op::OK);
                                }
                            }
                        }

                        future::ok(())
                    })
                }).into_future()
                .map(|_| ())
                .map_err(|_| ()),
        );

        Ok(())
    }

    #[test]
    fn can_connect_raw() {
        elog!();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let connect_cmd = ConnectCommandBuilder::default().build().unwrap();
        let options = NatsClientOptionsBuilder::default()
            .connect_command(connect_cmd)
            .cluster_uri("127.0.0.1:4222")
            .build()
            .unwrap();

        let connection = NatsClient::from_options(options);
        let (tx, rx) = oneshot::channel();
        runtime.spawn(connection.then(|r| tx.send(r).map_err(|e| panic!("Cannot send Result {:?}", e))));
        let connection_result = rx.wait().expect("Cannot wait for a result");
        let _ = runtime.shutdown_now().wait();
        debug!(target: "nitox", "can_connect_raw::connection_result {:#?}", connection_result);
        assert!(connection_result.is_ok());
    }

    #[test]
    fn can_connect() {
        elog!();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let connect_cmd = ConnectCommandBuilder::default().build().unwrap();
        let options = NatsClientOptionsBuilder::default()
            .connect_command(connect_cmd)
            .cluster_uri("127.0.0.1:4222")
            .build()
            .unwrap();

        let connection = NatsClient::from_options(options).and_then(|client| client.connect());
        let (tx, rx) = oneshot::channel();
        runtime.spawn(connection.then(|r| tx.send(r).map_err(|e| panic!("Cannot send Result {:?}", e))));
        let connection_result = rx.wait().expect("Cannot wait for a result");
        let _ = runtime.shutdown_now().wait();
        debug!(target: "nitox", "can_connect::connection_result {:#?}", connection_result);
        assert!(connection_result.is_ok());
    }

    #[test]
    fn can_sub_and_pub() {
        elog!();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let connect_cmd = ConnectCommandBuilder::default().build().unwrap();
        let options = NatsClientOptionsBuilder::default()
            .connect_command(connect_cmd)
            .cluster_uri("127.0.0.1:4222")
            .build()
            .unwrap();

        let fut = NatsClient::from_options(options)
            .and_then(|client| client.connect())
            .and_then(|client| {
                client
                    .subscribe(SubCommandBuilder::default().subject("foo").build().unwrap())
                    .map_err(|_| NatsError::InnerBrokenChain)
                    .and_then(move |stream| {
                        let _ = client
                            .publish(
                                PubCommandBuilder::default()
                                    .subject("foo")
                                    .payload("bar")
                                    .build()
                                    .unwrap(),
                            ).wait();

                        stream
                            .take(1)
                            .into_future()
                            .map(|(maybe_message, _)| maybe_message.unwrap())
                            .map_err(|_| NatsError::InnerBrokenChain)
                    })
            });

        let (tx, rx) = oneshot::channel();
        runtime.spawn(fut.then(|r| tx.send(r).map_err(|e| panic!("Cannot send Result {:?}", e))));
        let connection_result = rx.wait().expect("Cannot wait for a result");
        let _ = runtime.shutdown_now().wait();
        debug!(target: "nitox", "can_sub_and_pub::connection_result {:#?}", connection_result);
        assert!(connection_result.is_ok());
        let msg = connection_result.unwrap();
        assert_eq!(msg.payload, "bar");
    }

    #[test]
    #[allow(dead_code)]
    fn can_request() {
        elog!();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();

        let tcp_res = create_tcp_mock(&mut runtime, 1339, None);
        debug!(target: "nitox", "can_request::tcp_result {:#?}", tcp_res);
        assert!(tcp_res.is_ok());

        let connect_cmd = ConnectCommandBuilder::default().build().unwrap();
        let options = NatsClientOptionsBuilder::default()
            .connect_command(connect_cmd)
            .cluster_uri("127.0.0.1:1339")
            .build()
            .unwrap();

        let fut = NatsClient::from_options(options)
            .and_then(|client| client.connect())
            .and_then(|client| client.request("foo2".into(), "foo".into()));

        let (tx, rx) = oneshot::channel();
        runtime.spawn(fut.then(|r| tx.send(r).map_err(|e| panic!("Cannot send Result {:?}", e))));
        let connection_result = rx.wait().expect("Cannot wait for a result");
        let _ = runtime.shutdown_now().wait();
        debug!("can_request::connection_result {:#?}", connection_result);
        assert!(connection_result.is_ok());
        let msg = connection_result.unwrap();
        debug!("can_request::msg {:#?}", msg);
        assert_eq!(msg.payload, "bar");
    }

    #[test]
    fn can_ping_to_pong() {
        elog!();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let connect_cmd = ConnectCommandBuilder::default().build().unwrap();
        let options = NatsClientOptionsBuilder::default()
            .connect_command(connect_cmd)
            .cluster_uri("127.0.0.1:4222")
            .build()
            .unwrap();

        let fut = NatsClient::from_options(options)
            .and_then(|client| client.connect())
            .and_then(|client| client.send(Op::PING))
            .and_then(|client| {
                client
                    .skip_while(|op| future::ok(*op != Op::PONG))
                    .take(1)
                    .into_future()
                    .map(|(op, _)| op.unwrap())
                    .map_err(|(e, _)| e)
            });

        let (tx, rx) = oneshot::channel();
        runtime.spawn(fut.then(|r| tx.send(r).map_err(|e| panic!("Cannot send Result {:?}", e))));
        let connection_result = rx.wait().expect("Cannot wait for a result");
        let _ = runtime.shutdown_now().wait();
        debug!(target: "nitox", "can_ping_to_pong::connection_result {:#?}", connection_result);
        assert!(connection_result.is_ok());
        let op = connection_result.unwrap();
        assert_eq!(op, Op::PONG);
    }

    //#[test]
    #[allow(dead_code)]
    fn can_pong_to_ping() {
        elog!();
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let tcp_res = create_tcp_mock(&mut runtime, 1338, None);
        debug!(target: "nitox", "can_pong_to_ping::tcp_result {:#?}", tcp_res);
        assert!(tcp_res.is_ok());

        let connect_cmd = ConnectCommandBuilder::default().build().unwrap();
        let options = NatsClientOptionsBuilder::default()
            .connect_command(connect_cmd)
            .cluster_uri("127.0.0.1:1338")
            .build()
            .unwrap();

        let fut = NatsClient::from_options(options)
            .and_then(|client| client.connect())
            .and_then(|client| client.request("foo2".into(), "bar".into()));

        let (tx, rx) = oneshot::channel();
        runtime.spawn(fut.then(|r| tx.send(r).map_err(|e| panic!("Cannot send Result {:?}", e))));
        let connection_result = rx.wait().expect("Cannot wait for a result");
        let _ = runtime.shutdown_now().wait();
        debug!(target: "nitox", "can_pong_to_ping::connection_result {:#?}", connection_result);
        assert!(connection_result.is_ok());
    }
}
