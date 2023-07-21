use crate::packets::RejectionCode;
use crate::Error;
use sqlx::SqlitePool;
use tq_network::IntoErrorPacket;

/// Account information for a registered player. The account server uses this
/// information to authenticate the player on login. Passwords are hashed using
/// bcrypt
#[derive(Debug, sqlx::FromRow)]
pub struct Account {
    pub account_id: i32,
    pub username: String,
    pub password: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

impl Account {
    pub async fn auth(
        pool: &SqlitePool,
        username: &str,
        password: &str,
    ) -> Result<Account, Error> {
        let maybe_account = sqlx::query_as::<_, Self>(
            "SELECT * FROM accounts WHERE username = ?;",
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;
        match maybe_account {
            Some(account) => {
                let matched = bcrypt::verify(password, &account.password)?;
                if matched {
                    Ok(account)
                } else {
                    Err(RejectionCode::InvalidPassword
                        .packet()
                        .error_packet()
                        .into())
                }
            },
            None => Err(RejectionCode::InvalidPassword
                .packet()
                .error_packet()
                .into()),
        }
    }
}
