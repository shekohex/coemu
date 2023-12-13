use thiserror::Error;

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

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::SendError
    }
}
