use serde_json as json;

#[derive(Debug, Fail)]
pub enum CommandError {
    #[fail(display = "JSONError: {}", _0)]
    JsonError(json::Error),
    #[fail(display = "ValidationError: {}", _0)]
    ValidationError(ArgumentValidationError),
}

from_error!(json::Error, CommandError, CommandError::JsonError);
from_error!(
    ArgumentValidationError,
    CommandError,
    CommandError::ValidationError
);

#[derive(Debug, Clone, Eq, PartialEq, Fail)]
pub enum ArgumentValidationError {
    #[fail(display = "The argument contains spaces")]
    ContainsSpace,
    #[fail(display = "The argument contains tabs")]
    ContainsTab,
}
