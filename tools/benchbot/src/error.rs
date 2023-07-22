#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] auth::Error),
    #[error(transparent)]
    Network(#[from] tq_network::Error),
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
    #[error("Realm not found")]
    RealmNotFound,
    #[error("Server timed out")]
    ServerTimedOut,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Character name already taken")]
    CharacterNameAlreadyTaken,
    #[error("Token not found")]
    AccountTokenNotFound,
}
