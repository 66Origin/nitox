use futures::Future;
use native_tls::TlsConnector as NativeTlsConnector;
use std::net::SocketAddr;
use tokio_codec::{Decoder, Framed};
use tokio_tcp::TcpStream;
use tokio_tls::{TlsConnector, TlsStream};

use codec::OpCodec;
use error::NatsError;

pub type NatsConnection = Framed<TcpStream, OpCodec>;
pub type TLSNatsConnection = Framed<TlsStream<TcpStream>, OpCodec>;

pub(crate) fn connect(addr: &SocketAddr) -> impl Future<Item = NatsConnection, Error = NatsError> {
    TcpStream::connect(addr)
        .from_err()
        .map(move |socket| OpCodec::default().framed(socket))
}

pub(crate) fn connect_tls(host: String, addr: &SocketAddr) -> impl Future<Item = TLSNatsConnection, Error = NatsError> {
    let tls_connector = NativeTlsConnector::builder().build().unwrap();
    let tls_stream: TlsConnector = tls_connector.into();

    TcpStream::connect(addr)
        .from_err()
        .and_then(move |socket| tls_stream.connect(&host, socket).map_err(|e| e.into()))
        .map(|socket| OpCodec::default().framed(socket))
}
