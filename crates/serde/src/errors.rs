//! Handle Errors.
use core::fmt::Display;
use serde::{de, ser};

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

/// Represents Any Errors that could happens while Serializing/Deserializing
/// Binary Packets.
#[derive(Clone, Debug)]
pub enum TQSerdeError {
    Message(String),
    Utf8Error(core::str::Utf8Error),
    InvalidBool,
    Eof,
    DeserializeAnyNotSupported,
    Unspported,
}

impl Display for TQSerdeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TQSerdeError::Message(msg) => write!(f, "{}", msg),
            TQSerdeError::Utf8Error(err) => write!(f, "{}", err),
            TQSerdeError::InvalidBool => write!(f, "Invalid Boolean Value"),
            TQSerdeError::Eof => write!(f, "EOF"),
            TQSerdeError::DeserializeAnyNotSupported => {
                write!(f, "Deserializing Any Not Supported")
            },
            TQSerdeError::Unspported => write!(f, "Unspported Type"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TQSerdeError {}

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
