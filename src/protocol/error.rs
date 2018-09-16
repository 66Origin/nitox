use serde_json as json;

#[derive(Debug, Fail)]
pub enum CommandError {
    #[fail(display = "JSONError: {}", _0)]
    JsonError(json::Error),
    #[fail(display = "ValidationError: {}", _0)]
    ValidationError(ArgumentValidationError),
    #[fail(display = "Command is incomplete, cannot parse")]
    IncompleteCommandError,
    #[fail(display = "Command doesn't exist or is not supported")]
    CommandNotFoundOrSupported,
    #[fail(display = "Command is malformed")]
    CommandMalformed,
    #[fail(display = "UTF8Error: {}", _0)]
    UTF8SliceError(::std::str::Utf8Error),
    #[fail(display = "UTF8Error: {}", _0)]
    UTF8StringError(::std::string::FromUtf8Error),
    #[fail(display = "PayloadLengthParseError: {}", _0)]
    PayloadLengthParseError(::std::num::ParseIntError),
    #[fail(display = "GenericError: {}", _0)]
    GenericError(String),
}

from_error!(json::Error, CommandError, CommandError::JsonError);
from_error!(ArgumentValidationError, CommandError, CommandError::ValidationError);
from_error!(::std::str::Utf8Error, CommandError, CommandError::UTF8SliceError);
from_error!(
    ::std::string::FromUtf8Error,
    CommandError,
    CommandError::UTF8StringError
);
from_error!(
    ::std::num::ParseIntError,
    CommandError,
    CommandError::PayloadLengthParseError
);
from_error!(String, CommandError, CommandError::GenericError);

#[derive(Debug, Clone, Eq, PartialEq, Fail)]
pub enum ArgumentValidationError {
    #[fail(display = "The argument contains spaces")]
    ContainsSpace,
    #[fail(display = "The argument contains tabs")]
    ContainsTab,
}
