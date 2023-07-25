use crate::world::{Character, Map};
use crate::Error;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

mod actor_state;
mod token_store;

pub use actor_state::ActorState;

type Maps = Arc<HashMap<u32, Map>>;
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

        debug!("Loading Maps from Database");
        let db_maps = tq_db::map::Map::load_all(&pool).await?;
        let mut maps = HashMap::with_capacity(db_maps.len());
        debug!("Loaded #{} Map From Database", maps.len());
        for map in db_maps {
            let portals =
                tq_db::portal::Portal::by_map(&pool, map.map_id).await?;
            let map = Map::new(map, portals);
            maps.insert(map.id(), map);
        }
        let state = Self {
            token_store: token_store::TokenStore::new(),
            maps: Arc::new(maps),
            characters: Default::default(),
            pool,
        };
        Ok(state)
    }

    /// Get access to the database pool
    pub fn pool(&self) -> &SqlitePool { &self.pool }

    pub fn token_store(&self) -> &token_store::TokenStore { &self.token_store }

    pub fn maps(&self) -> &Maps { &self.maps }

    pub fn characters(&self) -> &Characters { &self.characters }

    /// Cleanup the state, close all connections and updating the database.
    pub async fn clean_up(self) -> Result<(), Error> {
        debug!("Clean up ..");
        debug!("Saving Characters data ..");
        let mut characters = self.characters().write().await;
        for (_, character) in characters.drain() {
            character.save(&self).await?;
        }
        self.pool().close().await;
        debug!("Closed Database Connection ..");
        Ok(())
    }
}
