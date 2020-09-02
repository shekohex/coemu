use crate::{packets::RejectionCode, Error, State};
use chrono::{DateTime, Utc};
use sqlx::types::ipnetwork::IpNetwork;
use tq_network::IntoErrorPacket;

/// Account information for a registered player. The account server uses this
/// information to authenticate the player on login. Passwords are hashed using
/// bcrypt
#[derive(Debug)]
pub struct Account {
    pub account_id: i32,
    pub username: String,
    pub password: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub ip_address: Option<IpNetwork>,
    pub created_at: DateTime<Utc>,
}

impl Account {
    pub async fn auth(
        username: &str,
        password: &str,
    ) -> Result<Account, Error> {
        let pool = State::global()?.pool();
        let account = sqlx::query_as!(
            Self,
            "SELECT * FROM accounts WHERE username = $1",
            username
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            RejectionCode::InvalidPassword.packet().error_packet()
        })?;

        let matched = bcrypt::verify(password, &account.password)?;
        if matched {
            Ok(account)
        } else {
            Err(RejectionCode::InvalidPassword
                .packet()
                .error_packet()
                .into())
        }
    }
}
