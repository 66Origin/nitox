use futures::Future;
//use native_tls::TlsConnector as NativeTlsConnector;
use std::net::SocketAddr;
use tokio_codec::{Decoder, Framed};
//use tokio_io::{AsyncRead, AsyncWrite};
use tokio_tcp::TcpStream;
//use tokio_tls::{TlsConnector, TlsStream};

use codec::OpCodec;
use error::NatsError;

pub type NatsConnection = Framed<TcpStream, OpCodec>;
//pub type TLSNatsConnection = Framed<TlsStream<TcpStream>, OpCodec>;

pub fn connect(addr: &SocketAddr) -> impl Future<Item = NatsConnection, Error = NatsError> {
    TcpStream::connect(addr)
        .from_err()
        .map(move |socket| OpCodec::default().framed(socket))
}

// FIXME: Halp, will do later, since I don't need it personally
/*pub fn connect_tls(host: &str, addr: &SocketAddr) -> impl Future<Item = TLSNatsConnection, Error = NatsError> {
    TcpStream::connect(addr)
        .from_err::<NatsError>()
        .map(move |socket| {
            let tls_stream: TlsConnector = match NativeTlsConnector::builder().build() {
                Ok(b) => b,
                Err(e) => return Err(e),
            }.into();

            Ok(tls_stream.connect(host, socket))
        }).and_then(move |maybe_socket| match maybe_socket {
            Ok(socket) => Ok(OpCodec::default().framed(socket)),
            Err(e) => Err(e.into()),
        }).map_err(|e| e.into())
}*/
