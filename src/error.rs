use std::{num::ParseIntError, str::Utf8Error};

use thiserror::Error;
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("request should contain valid UTF-8 characters")]
    InvalidUTF,
    #[error("request has invalid format")]
    InvalidFormat,
    #[error("unknown value")]
    UnknownValue,
}

#[derive(Debug, Error)]
pub enum RedisError {
    #[error("Command Parse Error: {0}")]
    CommandParseError(ParseError),

    #[error("Bulk String Parse Error: {0}")]
    BulkStringParseError(ParseError),

    #[error("Simple String Parse Error: {0}")]
    SimpleStringParseError(ParseError),

    #[error("Simple Error Parse Error: {0}")]
    SimpleErrorParseError(ParseError),

    #[error("Integer Parse Error: {0}")]
    IntegerParseError(ParseError),

    #[error("Boolean Parse Error: {0}")]
    BooleanParseError(ParseError),

    #[error("Nulls Parse Error: {0}")]
    NullsParseError(ParseError),

    #[error("Redis Value Parse Error: {0}")]
    ValueParseError(ParseError),

    #[error("index is out of bound")]
    IndexOutOfBoundError,

    #[error("CRLF not found")]
    CRLFNotFoundError,

    #[error("Unknown command")]
    UnknownCommand,

    #[error("Key does not exist")]
    KeyDoesNotExist,

    #[error("Key has been expired")]
    ExpiredKey,
}

pub type Result<T> = std::result::Result<T, RedisError>;

impl From<ParseIntError> for RedisError {
    fn from(_: ParseIntError) -> Self {
        RedisError::CommandParseError(ParseError::InvalidFormat)
    }
}
impl From<Utf8Error> for RedisError {
    fn from(_: Utf8Error) -> Self {
        RedisError::CommandParseError(ParseError::InvalidUTF)
    }
}
