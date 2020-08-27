use smol::channel::SendError;
use std::{backtrace::Backtrace, option::NoneError};
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
    #[error("NullError")]
    NullError {
        e: NoneError,
        #[backtrace]
        backtrace: Backtrace,
    },
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Self { Self::SendError }
}

impl From<NoneError> for Error {
    fn from(e: NoneError) -> Self {
        Error::NullError {
            e,
            backtrace: Backtrace::capture(),
        }
    }
}
