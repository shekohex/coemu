use bytes::Bytes;
use thiserror::Error;
use tq_network::{ErrorPacket, PacketEncode};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] tq_network::Error),
    #[error(transparent)]
    Server(#[from] tq_server::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    DotEnv(#[from] dotenvy::Error),
    #[error(transparent)]
    Env(#[from] std::env::VarError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error(transparent)]
    Db(#[from] tq_db::Error),
    #[error("State Error: {}", _0)]
    State(&'static str),
    #[error("{}", _0)]
    Other(String),
    #[error("Msg {}", _0)]
    Msg(u16, Bytes),
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
