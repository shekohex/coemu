use crate::{db, Error};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::Arc;

static STATE: OnceCell<State> = OnceCell::new();
type LoginTokens = Arc<DashMap<u32, (u32, u32)>>;
type CreationTokens = Arc<DashMap<u32, (u32, u32)>>;
type Clients = Arc<DashMap<usize, ClientState>>;
#[derive(Debug, Clone)]
pub struct State {
    clients: Clients,
    login_tokens: LoginTokens,
    creation_tokens: CreationTokens,
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
        let state = Self {
            clients: Arc::new(DashMap::new()),
            login_tokens: Arc::new(DashMap::new()),
            creation_tokens: Arc::new(DashMap::new()),
            pool,
        };
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

    pub fn clients(&self) -> &Clients { &self.clients }

    /// Get access to the database pool
    pub fn pool(&self) -> &PgPool { &self.pool }

    pub fn login_tokens(&self) -> &LoginTokens { &self.login_tokens }

    pub fn creation_tokens(&self) -> &CreationTokens { &self.creation_tokens }
}

#[derive(Debug)]
pub struct ClientState {
    pub character: db::Character,
}
