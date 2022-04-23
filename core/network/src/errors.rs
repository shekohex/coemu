use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    TQSerde(#[from] tq_serde::TQSerdeError),
    #[error("Actor Send Error!")]
    SendError,
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("{}", _0)]
    Other(String),
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self {
        Self::SendError
    }
}
