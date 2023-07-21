#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("Account not found")]
    AccountNotFound,
    #[error("Invalid password")]
    InvalidPassword,
}
