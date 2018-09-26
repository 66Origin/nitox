use bytes::Bytes;

pub trait Command {
    const CMD_NAME: &'static [u8];
    fn into_vec(self) -> Result<Bytes, CommandError>;
    fn try_parse(buf: &[u8]) -> Result<Self, CommandError>
    where
        Self: Sized;
}

pub fn check_command_arg(s: &str) -> Result<(), ArgumentValidationError> {
    if s.contains(' ') {
        return Err(ArgumentValidationError::ContainsSpace);
    } else if s.contains('\t') {
        return Err(ArgumentValidationError::ContainsTab);
    }

    Ok(())
}

macro_rules! check_cmd_arg {
    ($val:ident, $part:expr) => {
        use protocol::{check_command_arg, ArgumentValidationError};

        match check_command_arg($val) {
            Ok(_) => {}
            Err(ArgumentValidationError::ContainsSpace) => {
                return Err(format!("{} contains spaces", $part).into());
            }
            Err(ArgumentValidationError::ContainsTab) => {
                return Err(format!("{} contains tabs", $part).into());
            }
        }
    };
}

mod error;
pub use self::error::*;

mod client;
mod server;

mod op;
pub use self::op::*;

pub mod commands {
    pub use super::{
        client::{connect::*, pub_cmd::*, sub_cmd::*, unsub_cmd::*},
        server::{info::*, message::*, server_error::ServerError},
    };
}
