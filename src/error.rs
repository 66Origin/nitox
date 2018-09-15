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
    #[fail(display = "GenericError: {}", _0)]
    GenericError(String),
}

from_error!(::std::io::Error, NatsError, NatsError::IOError);
from_error!(protocol::CommandError, NatsError, NatsError::ProtocolError);
from_error!(String, NatsError, NatsError::GenericError);
