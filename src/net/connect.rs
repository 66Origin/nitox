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
    Tcp(Framed<TcpStream, OpCodec>),
    Tls(Framed<TlsStream<TcpStream>, OpCodec>),
}

impl Sink for NatsConnection {
    type SinkItem = Op;
    type SinkError = NatsError;

    fn start_send(&mut self, item: Op) -> StartSend<Self::SinkItem, Self::SinkError> {
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
    type Item = Op;
    type Error = NatsError;

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
        .and_then(move |socket| future::ok(NatsConnection::Tcp(OpCodec::default().framed(socket))))
}

pub(crate) fn connect_tls(host: String, addr: &SocketAddr) -> impl Future<Item = NatsConnection, Error = NatsError> {
    let tls_connector = NativeTlsConnector::builder().build().unwrap();
    let tls_stream: TlsConnector = tls_connector.into();

    TcpStream::connect(addr)
        .from_err()
        .and_then(move |socket| tls_stream.connect(&host, socket).map_err(|e| e.into()))
        .and_then(|socket| future::ok(NatsConnection::Tls(OpCodec::default().framed(socket))))
}
