use crate::world::{Character, Map};
use crate::{db, Error};
use once_cell::sync::OnceCell;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

mod actor_state;
mod token_store;

pub use actor_state::ActorState;

/// Global state for the game server.
static STATE: OnceCell<State> = OnceCell::new();

type Maps = Arc<RwLock<HashMap<u32, Map>>>;
type Characters = Arc<RwLock<HashMap<u32, Character>>>;
type Shared<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone)]
pub struct State {
    token_store: token_store::TokenStore,
    characters: Characters,
    maps: Maps,
    pool: SqlitePool,
}

impl State {
    /// Init The State.
    /// Should only get called once.
    pub async fn init() -> Result<(), Error> {
        let data_dir = dotenv::var("DATA_LOCATION")?;
        let default_db_location =
            format!("sqlite://{data_dir}/coemu.db?mode=rwc");
        let db_url = dotenv::var("DATABASE_URL").unwrap_or(default_db_location);
        let pool = SqlitePoolOptions::new()
            .max_connections(42)
            .min_connections(4)
            .connect(&db_url)
            .await?;
        let state = Self {
            token_store: token_store::TokenStore::new(),
            maps: Default::default(),
            characters: Default::default(),
            pool,
        };
        STATE
            .set(state)
            .map_err(|_| Error::State("Failed to init the state."))?;
        Self::post_init().await?;
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
    pub fn pool(&self) -> &SqlitePool { &self.pool }

    pub fn token_store(&self) -> &token_store::TokenStore { &self.token_store }

    pub fn maps(&self) -> &Maps { &self.maps }

    pub fn characters(&self) -> &Characters { &self.characters }

    /// Cleanup the state, close all connections and updating the database.
    pub async fn clean_up() -> Result<(), Error> {
        debug!("Clean up ..");
        let state = Self::global()?;
        debug!("Saving Characters data ..");
        let mut characters = state.characters().write().await;
        for (_, character) in characters.drain() {
            character.save().await?;
        }
        state.pool().close().await;
        debug!("Closed Database Connection ..");
        Ok(())
    }

    /// For Things we should do before sending that we init the state
    async fn post_init() -> Result<(), Error> {
        let state = Self::global()?;
        state.init_maps().await?;
        Ok(())
    }

    /// This method loads the compressed conquer maps from the flat-file
    /// database using the database's maps table.
    async fn init_maps(&self) -> Result<(), Error> {
        debug!("Loading Maps from Database");
        let maps = db::Map::load_all().await?;
        debug!("Loaded #{} Map From Database", maps.len());
        let mut lock = self.maps.write().await;
        for map in maps {
            let portals = db::Portal::by_map(map.map_id).await?;
            let map = Map::new(map, portals);
            lock.insert(map.id(), map);
        }
        Ok(())
    }
}
