//! Handle Errors.
use serde::{de, ser};
use std::fmt::Display;
use thiserror::Error;

/// Represents Any Errors that could happens while Serializing/Deserializing
/// Binary Packets.
#[derive(Clone, Debug, Error)]
pub enum TQSerdeError {
    #[error("{}", _0)]
    Message(String),
    #[error("Invalid Boolean Value")]
    InvalidBool,
    #[error("EOF")]
    Eof,
    #[error("Deserializing Any Not Supported")]
    DeserializeAnyNotSupported,
    #[error("Unspported Type")]
    Unspported,
}

impl ser::Error for TQSerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        TQSerdeError::Message(msg.to_string())
    }
}

impl de::Error for TQSerdeError {
    fn custom<T: Display>(msg: T) -> Self {
        TQSerdeError::Message(msg.to_string())
    }
}
