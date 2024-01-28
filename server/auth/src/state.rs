use crate::error::Error;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

#[derive(Debug, Clone)]
pub struct State {
    pool: SqlitePool,
}

impl State {
    /// Init The State.
    /// Should only get called once.
    pub async fn init() -> Result<Self, Error> {
        let data_dir = dotenvy::var("DATA_LOCATION")?;
        let default_db_location = format!("sqlite://{data_dir}/coemu.db?mode=rwc");
        let db_url = dotenvy::var("DATABASE_URL").unwrap_or(default_db_location);
        let pool = SqlitePoolOptions::new()
            .max_connections(42)
            .min_connections(4)
            .connect(&db_url)
            .await?;
        let state = Self { pool };
        Ok(state)
    }

    /// Get access to the database pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
