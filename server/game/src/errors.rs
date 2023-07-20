use bytes::Bytes;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot};
use tq_network::{ErrorPacket, PacketEncode};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] tq_network::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    DotEnv(#[from] dotenvy::Error),
    #[error(transparent)]
    Env(#[from] std::env::VarError),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("State Error: {}", _0)]
    State(&'static str),
    #[error("Channel Send Error!")]
    SendError,
    #[error("Channel Recv Error!")]
    RecvError,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseFloat(#[from] std::num::ParseFloatError),
    #[error("{}", _0)]
    Other(String),
    #[error("Msg {}", _0)]
    Msg(u16, Bytes),
    #[error("Map Region not found!")]
    MapRegionNotFound,
}

impl<T> From<mpsc::error::SendError<T>> for Error {
    fn from(_: mpsc::error::SendError<T>) -> Self { Self::SendError }
}

impl From<oneshot::error::RecvError> for Error {
    fn from(_: oneshot::error::RecvError) -> Self { Self::RecvError }
}

impl<T: PacketEncode> From<ErrorPacket<T>> for Error {
    fn from(v: ErrorPacket<T>) -> Self {
        let (id, bytes) = v.0.encode().unwrap();
        Self::Msg(id, bytes)
    }
}

impl PacketEncode for Error {
    type Error = Self;
    type Packet = ();

    fn encode(&self) -> Result<(u16, Bytes), Self::Error> {
        match self {
            Self::Msg(id, bytes) => Ok((*id, bytes.clone())),
            e => Err(Self::Other(e.to_string())),
        }
    }
}
