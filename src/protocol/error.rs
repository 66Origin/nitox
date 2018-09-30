use serde_json as json;

/// This error is designed to wrap all the possible errors tha can occur during decoding/parsing and encoding of any command
#[derive(Debug, Fail)]
pub enum CommandError {
    /// The given JSON was invalid and/or malformed
    #[fail(display = "JSONError: {}", _0)]
    JsonError(json::Error),
    /// A validation of the arguments failed, meaning improper arguments were given to it
    #[fail(display = "ValidationError: {}", _0)]
    ValidationError(ArgumentValidationError),
    /// Occurs when a command is incomplete and we cannot parse it yet without more from the buffer
    #[fail(display = "Command is incomplete, cannot parse")]
    IncompleteCommandError,
    /// Occurs when a command name that doesn't exist (or is not supported as of now) is given to the parser
    #[fail(display = "Command doesn't exist or is not supported")]
    CommandNotFoundOrSupported,
    /// Occurs when a command is malformed, whether it's order/conformance of fields (for instance, a mismatch
    /// between `payload_len` and the length of the actual payload in PUB and MSG commands trigger this error)
    #[fail(display = "Command is malformed")]
    CommandMalformed,
    /// Occurs if we try to parse a piece of command (string slice aka `str` only) that is supposed to be valid UTF8 and...is actually not
    #[fail(display = "UTF8Error: {}", _0)]
    UTF8SliceError(::std::str::Utf8Error),
    /// Occurs if we try to parse a piece of command (owned string aka `String` only) that is supposed to be valid UTF8 and...is actually not
    #[fail(display = "UTF8Error: {}", _0)]
    UTF8StringError(::std::string::FromUtf8Error),
    /// Occurs when the payload length exceeds the bounds of integers
    #[fail(display = "PayloadLengthParseError: {}", _0)]
    PayloadLengthParseError(::std::num::ParseIntError),
    /// Generic error for untyped `String` errors
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

/// This error is designed to be given when an argument like the `subject` or `queue_group` arguments are
/// containing spaces or tabs, which is prohibited by the protocol and trigger an error server-side
#[derive(Debug, Clone, Eq, PartialEq, Fail)]
pub enum ArgumentValidationError {
    /// The argument contains spaces
    #[fail(display = "The argument contains spaces")]
    ContainsSpace,
    /// The argument contains tabs
    #[fail(display = "The argument contains tabs")]
    ContainsTab,
}
