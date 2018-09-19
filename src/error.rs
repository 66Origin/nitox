use super::protocol;

macro_rules! from_error {
    ($type:ty, $target:ident, $targetvar:expr) => {
        impl From<$type> for $target {
            fn from(s: $type) -> Self {
                $targetvar(s.into())
            }
        }
    };
}

#[derive(Debug, Fail)]
pub enum NatsError {
    #[fail(display = "IOError: {}", _0)]
    IOError(::std::io::Error),
    #[fail(display = "ProtocolError: {}", _0)]
    ProtocolError(protocol::CommandError),
    #[fail(display = "UTF8Error: {}", _0)]
    UTF8Error(::std::string::FromUtf8Error),
    #[fail(display = "TlsError: {}", _0)]
    TlsError(::native_tls::Error),
    #[fail(display = "TlsHostMissingError: Host is missing, can't verify server identity")]
    TlsHostMissingError,
    #[fail(display = "UrlParseError: {}", _0)]
    UrlParseError(::url::ParseError),
    #[fail(display = "AddrParseError: {}", _0)]
    AddrParseError(::std::net::AddrParseError),
    #[fail(display = "InnerBrokenChain: the sender/receiver pair has been disconnected")]
    InnerBrokenChain,
    #[fail(display = "GenericError: {}", _0)]
    GenericError(String),
}

from_error!(::std::io::Error, NatsError, NatsError::IOError);
from_error!(protocol::CommandError, NatsError, NatsError::ProtocolError);
from_error!(::std::string::FromUtf8Error, NatsError, NatsError::UTF8Error);
from_error!(::native_tls::Error, NatsError, NatsError::TlsError);
from_error!(String, NatsError, NatsError::GenericError);
from_error!(::url::ParseError, NatsError, NatsError::UrlParseError);
from_error!(::std::net::AddrParseError, NatsError, NatsError::AddrParseError);

impl<T> From<::futures::sync::mpsc::SendError<T>> for NatsError {
    fn from(_: ::futures::sync::mpsc::SendError<T>) -> Self {
        NatsError::InnerBrokenChain
    }
}
