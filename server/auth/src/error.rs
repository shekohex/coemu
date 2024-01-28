use bytes::Bytes;
use tq_network::{ErrorPacket, PacketEncode};

#[derive(Debug)]
pub enum Error {
    Wasmtime(wasmtime::Error),
    Network(tq_network::Error),
    #[cfg(feature = "server")]
    Server(tq_server::Error),
    IO(std::io::Error),
    DotEnv(dotenvy::Error),
    Env(std::env::VarError),
    Sqlx(sqlx::Error),
    Db(tq_db::Error),
    State(&'static str),
    Other(String),
    Msg(u16, Bytes),
    ActorNotFound,
    InvalidPacket,
}

impl From<tq_db::Error> for Error {
    fn from(v: tq_db::Error) -> Self {
        Self::Db(v)
    }
}

impl From<sqlx::Error> for Error {
    fn from(v: sqlx::Error) -> Self {
        Self::Sqlx(v)
    }
}

impl From<std::env::VarError> for Error {
    fn from(v: std::env::VarError) -> Self {
        Self::Env(v)
    }
}

impl From<dotenvy::Error> for Error {
    fn from(v: dotenvy::Error) -> Self {
        Self::DotEnv(v)
    }
}

impl From<std::io::Error> for Error {
    fn from(v: std::io::Error) -> Self {
        Self::IO(v)
    }
}

#[cfg(feature = "server")]
impl From<tq_server::Error> for Error {
    fn from(v: tq_server::Error) -> Self {
        Self::Server(v)
    }
}

impl From<tq_network::Error> for Error {
    fn from(v: tq_network::Error) -> Self {
        Self::Network(v)
    }
}

impl From<wasmtime::Error> for Error {
    fn from(v: wasmtime::Error) -> Self {
        Self::Wasmtime(v)
    }
}

impl std::error::Error for Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Wasmtime(e) => write!(f, "Wasmtime error: {}", e),
            Self::Network(e) => write!(f, "Network error: {}", e),
            #[cfg(feature = "server")]
            Self::Server(e) => write!(f, "Server error: {}", e),
            Self::IO(e) => write!(f, "IO error: {}", e),
            Self::DotEnv(e) => write!(f, "DotEnv error: {}", e),
            Self::Env(e) => write!(f, "Env error: {}", e),
            Self::Sqlx(e) => write!(f, "Sqlx error: {}", e),
            Self::Db(e) => write!(f, "Db error: {}", e),
            Self::State(e) => write!(f, "State error: {}", e),
            Self::Other(e) => write!(f, "{}", e),
            Self::Msg(id, bytes) => {
                write!(f, "Error packet: id = {}, body = {:?}", id, bytes)
            },
            Self::ActorNotFound => write!(f, "Actor Not Found"),
            Self::InvalidPacket => write!(f, "Invalid Packet"),
        }
    }
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
