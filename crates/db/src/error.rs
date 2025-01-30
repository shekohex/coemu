#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "sqlx")]
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("Account not found")]
    AccountNotFound,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Creating account failed")]
    CreateAccountFailed,
}
