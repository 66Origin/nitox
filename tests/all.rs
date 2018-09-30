#[macro_use]
extern crate log;
extern crate env_logger;
extern crate futures;
extern crate nitox;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_executor;
extern crate tokio_tcp;

use futures::{
    future,
    prelude::*,
    sync::{mpsc, oneshot},
};
use nitox::{codec::OpCodec, commands::*, NatsClient, NatsClientOptions, NatsError, Op};
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
                            debug!(target: "nitox", "Got PONG from client");
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
                            debug!(target: "nitox", "Got PUB command {:#?}", cmd);
                            if verbose {
                                let _ = tx.unbounded_send(Op::OK);
                            }
                            let mut builder = Message::builder();
                            let sub = cmd.subject.clone();
                            builder.subject(cmd.reply_to.unwrap_or(sub));
                            if let Ok(sid) = sid_lock.read() {
                                builder.sid((*sid).clone());
                            }
                            builder.payload("bar");

                            let msg = builder.build().unwrap();
                            debug!(target: "nitox", "Replying with MSG command {:#?}", msg);
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
    let connect_cmd = ConnectCommand::builder().build().unwrap();
    let options = NatsClientOptions::builder()
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
    let connect_cmd = ConnectCommand::builder().build().unwrap();
    let options = NatsClientOptions::builder()
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
    let connect_cmd = ConnectCommand::builder().build().unwrap();
    let options = NatsClientOptions::builder()
        .connect_command(connect_cmd)
        .cluster_uri("127.0.0.1:4222")
        .build()
        .unwrap();

    let fut = NatsClient::from_options(options)
        .and_then(|client| client.connect())
        .and_then(|client| {
            client
                .subscribe(SubCommand::builder().subject("foo").build().unwrap())
                .map_err(|_| NatsError::InnerBrokenChain)
                .and_then(move |stream| {
                    let _ = client
                        .publish(PubCommand::builder().subject("foo").payload("bar").build().unwrap())
                        .wait();

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
fn can_request() {
    elog!();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();

    let tcp_res = create_tcp_mock(&mut runtime, 1339, None);
    debug!(target: "nitox", "can_request::tcp_result {:#?}", tcp_res);
    assert!(tcp_res.is_ok());

    let connect_cmd = ConnectCommand::builder().build().unwrap();
    let options = NatsClientOptions::builder()
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
    let connect_cmd = ConnectCommand::builder().build().unwrap();
    let options = NatsClientOptions::builder()
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

#[test]
fn can_pong_to_ping() {
    elog!();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let tcp_res = create_tcp_mock(&mut runtime, 1338, None);
    debug!(target: "nitox", "can_pong_to_ping::tcp_result {:#?}", tcp_res);
    assert!(tcp_res.is_ok());

    let connect_cmd = ConnectCommand::builder().build().unwrap();
    let options = NatsClientOptions::builder()
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
