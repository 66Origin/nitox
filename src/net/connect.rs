use futures::{future, prelude::*};
use native_tls::TlsConnector as NativeTlsConnector;
use std::net::SocketAddr;
use tokio_codec::{Decoder, Framed};
use tokio_tcp::TcpStream;
use tokio_tls::{TlsConnector, TlsStream};

use codec::OpCodec;
use error::NatsError;
use protocol::Op;

#[derive(Debug)]
pub enum NatsConnection {
    Tcp(Box<Framed<TcpStream, OpCodec>>),
    Tls(Box<Framed<TlsStream<TcpStream>, OpCodec>>),
}

impl Sink for NatsConnection {
    type SinkError = NatsError;
    type SinkItem = Op;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        match self {
            NatsConnection::Tcp(framed) => framed.start_send(item),
            NatsConnection::Tls(framed) => framed.start_send(item),
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        match self {
            NatsConnection::Tcp(framed) => framed.poll_complete(),
            NatsConnection::Tls(framed) => framed.poll_complete(),
        }
    }
}

impl Stream for NatsConnection {
    type Error = NatsError;
    type Item = Op;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self {
            NatsConnection::Tcp(framed) => framed.poll(),
            NatsConnection::Tls(framed) => framed.poll(),
        }
    }
}

pub(crate) fn connect(addr: &SocketAddr) -> impl Future<Item = NatsConnection, Error = NatsError> {
    TcpStream::connect(addr)
        .from_err()
        .and_then(move |socket| future::ok(NatsConnection::Tcp(Box::new(OpCodec::default().framed(socket)))))
}

pub(crate) fn connect_tls(host: String, addr: &SocketAddr) -> impl Future<Item = NatsConnection, Error = NatsError> {
    let tls_connector = NativeTlsConnector::builder().build().unwrap();
    let tls_stream: TlsConnector = tls_connector.into();

    TcpStream::connect(addr)
        .from_err()
        .and_then(move |socket| tls_stream.connect(&host, socket).map_err(|e| e.into()))
        .and_then(|socket| future::ok(NatsConnection::Tls(Box::new(OpCodec::default().framed(socket)))))
}
