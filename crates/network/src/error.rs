#[cfg(not(feature = "std"))]
use alloc::string::String;

#[derive(Debug, Clone)]
pub enum Error {
    TQSerde(tq_serde::TQSerdeError),
    SendError,
    Other(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TQSerde(e) => write!(f, "TQSerde Error: {}", e),
            Self::SendError => write!(f, "Send Error"),
            Self::Other(e) => write!(f, "{}", e),
        }
    }
}

impl From<tq_serde::TQSerdeError> for Error {
    fn from(e: tq_serde::TQSerdeError) -> Self {
        Self::TQSerde(e)
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::SendError
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
