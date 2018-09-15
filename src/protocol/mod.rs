pub trait Command {
    fn into_vec(self) -> Result<Vec<u8>, CommandError>;
}

impl Command {
    fn check(s: &String) -> Result<(), ArgumentValidationError> {
        if s.contains(" ") {
            return Err(ArgumentValidationError::ContainsSpace);
        } else if s.contains("\t") {
            return Err(ArgumentValidationError::ContainsTab);
        }

        Ok(())
    }
}

macro_rules! check_cmd_arg {
    ($val:ident, $part:expr) => {
        use protocol::ArgumentValidationError;

        match Command::check($val) {
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

pub mod client;
pub mod op;
pub mod server;
