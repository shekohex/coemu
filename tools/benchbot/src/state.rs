use std::collections::HashMap;
use std::sync::Arc;

use crate::Error;
use parking_lot::RwLock;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

type Shared<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone)]
pub struct State {
    pool: SqlitePool,
    /// Maps Account ID to Token value
    tokens: Shared<HashMap<i32, u64>>,
}

impl State {
    /// Init The State.
    /// Should only get called once.
    pub async fn init() -> Result<Self, Error> {
        let data_dir = dotenvy::var("DATA_LOCATION")?;
        let default_db_location =
            format!("sqlite://{data_dir}/coemu.db?mode=rwc");
        let db_url =
            dotenvy::var("DATABASE_URL").unwrap_or(default_db_location);
        let pool = SqlitePoolOptions::new()
            .max_connections(42)
            .min_connections(4)
            .connect(&db_url)
            .await?;
        let state = Self {
            pool,
            tokens: Default::default(),
        };
        Ok(state)
    }

    /// Get access to the database pool
    pub fn pool(&self) -> &SqlitePool { &self.pool }

    pub fn tokens(&self) -> &Shared<HashMap<i32, u64>> { &self.tokens }
}
