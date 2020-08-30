use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] network::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Env(#[from] dotenv::Error),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error("State Error: {}", _0)]
    State(&'static str),
    #[error("{}", _0)]
    Other(&'static str),
}
