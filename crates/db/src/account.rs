/// Account information for a registered player. The account server uses this
/// information to authenticate the player on login. Passwords are hashed using
/// bcrypt
#[derive(Default, Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Account {
    pub account_id: i32,
    pub username: String,
    pub password: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[cfg(feature = "sqlx")]
impl Account {
    pub async fn auth(pool: &sqlx::SqlitePool, username: &str, password: &str) -> Result<Account, crate::Error> {
        let maybe_account = sqlx::query_as::<_, Self>("SELECT * FROM accounts WHERE username = ?;")
            .bind(username)
            .fetch_optional(pool)
            .await?;
        match maybe_account {
            Some(account) => {
                let matched = bcrypt::verify(password, &account.password)?;
                if matched {
                    Ok(account)
                } else {
                    Err(crate::Error::InvalidPassword)
                }
            },
            None => Err(crate::Error::AccountNotFound),
        }
    }

    /// Returns all accounts in the database.
    ///
    /// Useful for testing purposes.
    pub async fn all(
        pool: &sqlx::SqlitePool,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Self>, crate::Error> {
        use futures::TryFutureExt;
        sqlx::query_as::<_, Self>("SELECT * FROM accounts LIMIT ? OFFSET ?;")
            .bind(limit.unwrap_or(100))
            .bind(offset.unwrap_or(0))
            .fetch_all(pool)
            .map_err(Into::into)
            .await
    }

    // === Methods ===

    /// Creates a new account in the database.
    pub async fn create(mut self, pool: &sqlx::SqlitePool) -> Result<Self, crate::Error> {
        let password = bcrypt::hash(&self.password, bcrypt::DEFAULT_COST)?;
        let res = sqlx::query("INSERT INTO accounts (username, password, name, email) VALUES (?, ?, ?, ?);")
            .bind(&self.username)
            .bind(&password)
            .bind(&self.name)
            .bind(&self.email)
            .execute(pool)
            .await?;
        if res.rows_affected() == 0 {
            Err(crate::Error::CreateAccountFailed)
        } else {
            self.account_id = res.last_insert_rowid() as i32;
            Ok(self)
        }
    }
}
