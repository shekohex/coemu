use serde::{de, ser};
use std::fmt::Display;
use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum TQSerdeError {
    #[error("{}", _0)]
    Message(String),
    #[error("EOF")]
    Eof,
    #[error("Deserializing Any Not Supported")]
    DeserializeAnyNotSupported,
    #[error("Syntax Error")]
    Syntax,
    #[error("Expected Integer")]
    ExpectedInteger,
    #[error("Expected String")]
    ExpectedString,
    #[error("Sequence Must Have Length")]
    SequenceMustHaveLength,
    #[error("Bad Trailing Characters")]
    TrailingCharacters,
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
