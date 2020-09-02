use crate::{world::Character, Error};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::RwLock;

static STATE: OnceCell<State> = OnceCell::new();

type LoginTokens = Arc<DashMap<u32, (u32, u32)>>;
type CreationTokens = Arc<DashMap<u32, (u32, u32)>>;
type Shared<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone)]
pub struct State {
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

    /// Get access to the database pool
    pub fn pool(&self) -> &PgPool { &self.pool }

    pub fn login_tokens(&self) -> &LoginTokens { &self.login_tokens }

    pub fn creation_tokens(&self) -> &CreationTokens { &self.creation_tokens }
}

#[derive(Debug, Default, Clone)]
pub struct ActorState {
    character: Shared<Character>,
}

impl ActorState {
    pub async fn character(&self) -> impl Deref<Target = Character> + '_ {
        self.character.read().await
    }

    pub async fn character_mut(
        &self,
    ) -> impl DerefMut<Target = Character> + '_ {
        self.character.write().await
    }
}
