#[derive(Debug)]
pub enum Error {
    TQNetwork(tq_network::Error),
    AddrParseError(std::net::AddrParseError),
    IO(std::io::Error),
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TQNetwork(e) => write!(f, "TQNetwork Error: {}", e),
            Self::AddrParseError(e) => write!(f, "AddrParse Error: {}", e),
            Self::IO(e) => write!(f, "IO Error: {}", e),
            Self::Internal(s) => write!(f, "Internal Error: {}", s),
        }
    }
}

impl From<tq_network::Error> for Error {
    fn from(e: tq_network::Error) -> Self {
        Self::TQNetwork(e)
    }
}

impl From<std::net::AddrParseError> for Error {
    fn from(e: std::net::AddrParseError) -> Self {
        Self::AddrParseError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}

impl std::error::Error for Error {}
