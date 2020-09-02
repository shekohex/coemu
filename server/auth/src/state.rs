use crate::Error;
use once_cell::sync::OnceCell;
use sqlx::postgres::{PgPool, PgPoolOptions};

static STATE: OnceCell<State> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct State {
    pool: PgPool,
}

impl State {
    /// Init The State.
    /// Should only get called once.
    pub async fn init() -> Result<(), Error> {
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .min_connections(4)
            .test_before_acquire(true)
            .connect(&dotenv::var("DATABASE_URL")?)
            .await?;
        let state = Self { pool };
        STATE
            .set(state)
            .map_err(|_| Error::State("Failed to init the state."))?;
        Ok(())
    }

    /// Get access to the global state.
    pub fn global() -> Result<&'static Self, Error> {
        STATE.get().ok_or_else(|| {
            Error::State(
                "State is uninialized, did you forget to call State::init()!",
            )
        })
    }

    /// Get access to the database pool
    pub fn pool(&self) -> &PgPool { &self.pool }
}
