use std::fmt;

#[derive(Debug, Clone)]
pub struct ServerError(String);
impl From<String> for ServerError {
    fn from(s: String) -> Self {
        ServerError(s)
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub mod info;
pub mod message;
