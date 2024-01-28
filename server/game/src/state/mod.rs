use crate::entities::GameEntity;
use crate::world::Map;
use crate::Error;
use parking_lot::{Mutex, RwLock};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

mod actor_state;

pub use actor_state::ActorState;

type Maps = HashMap<u32, Map>;
type Entites = RwLock<HashMap<u32, Arc<GameEntity>>>;
type LoginTokens = Mutex<HashMap<u64, LoginToken>>;
type CreationTokens = Mutex<HashMap<u32, CreationToken>>;

#[derive(Debug)]
pub struct State {
    login_tokens: LoginTokens,
    creation_tokens: CreationTokens,
    entities: Entites,
    maps: Maps,
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
        Self::with_pool(pool).await
    }

    pub async fn with_pool(pool: SqlitePool) -> Result<Self, Error> {
        debug!("Loading Maps from Database");
        let db_maps = tq_db::map::Map::load_all(&pool).await?;
        let mut maps = HashMap::with_capacity(db_maps.len());
        debug!("Loaded #{} Map From Database", db_maps.len());
        for map in db_maps {
            let portals = tq_db::portal::Portal::by_map(&pool, map.id).await?;
            tracing::trace!(%map.id, portals = %portals.len(), "Loaded Portals");
            let npcs = tq_db::npc::Npc::by_map(&pool, map.id).await?;
            tracing::trace!(%map.id, npcs = %npcs.len(), "Loaded Npcs");
            let map = Map::new(map, portals, npcs);
            maps.insert(map.id(), map);
        }

        let state = Self {
            login_tokens: Default::default(),
            creation_tokens: Default::default(),
            entities: Default::default(),
            maps,
            pool,
        };
        Ok(state)
    }

    /// Get access to the database pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn maps(&self) -> &Maps {
        &self.maps
    }

    pub fn try_map(&self, map_id: u32) -> Result<&Map, Error> {
        self.maps.get(&map_id).ok_or(Error::MapNotFound)
    }

    pub fn insert_entity(&self, entity: Arc<GameEntity>) {
        let mut entities = self.entities.write();
        entities.insert(entity.id(), entity);
    }

    pub fn remove_entity(&self, id: u32) {
        let mut entities = self.entities.write();
        entities.remove(&id);
    }

    pub fn with_entity<F, R>(&self, id: u32, f: F) -> Option<R>
    where
        F: FnOnce(&GameEntity) -> R,
    {
        let entities = self.entities.read();
        entities.get(&id).map(|v| f(v))
    }

    pub fn entities(&self) -> Vec<Arc<GameEntity>> {
        let lock = self.entities.read();
        let values = lock.values();
        values.cloned().collect()
    }

    /// Generate a new Login Token.
    ///
    /// The token will be stored internally, and can be later removed by calling
    /// [`TokenStore::remove_login_token`].
    pub fn generate_login_token(&self, account_id: u32, realm_id: u32) -> Result<GeneratedLoginToken, crate::Error> {
        let token = rand::random();
        self.login_tokens
            .lock()
            .insert(token, LoginToken { account_id, realm_id });
        Ok(GeneratedLoginToken { token })
    }

    /// Remove a Login Token.
    pub fn remove_login_token(&self, token: u64) -> Result<LoginToken, crate::Error> {
        self.login_tokens
            .lock()
            .remove(&token)
            .ok_or(crate::Error::LoginTokenNotFound)
    }

    /// Store a new CreationToken.
    /// The token will be stored internally, and can be later removed by calling
    /// [`TokenStore::remove_creation_token`].
    pub fn store_creation_token(&self, token: u32, account_id: u32, realm_id: u32) -> Result<(), crate::Error> {
        self.creation_tokens
            .lock()
            .insert(token, CreationToken { account_id, realm_id });
        Ok(())
    }

    /// Remove a CreationToken.
    pub fn remove_creation_token(&self, token: u32) -> Result<CreationToken, crate::Error> {
        self.creation_tokens
            .lock()
            .remove(&token)
            .ok_or(crate::Error::CreationTokenNotFound)
    }

    fn drain_entities(&self) -> Vec<Arc<GameEntity>> {
        let mut entities = self.entities.write();
        let values = entities.drain();
        values.map(|(_, v)| v).collect()
    }

    /// Cleanup the state, close all connections and updating the database.
    pub async fn clean_up(self) -> Result<(), Error> {
        debug!("Clean up ..");
        debug!("Saving Entities data ..");
        let entities = self.drain_entities();
        for e in entities {
            match e.as_ref() {
                GameEntity::Character(character) => character.save(&self).await?,
                GameEntity::Npc(_) => {
                    // Do nothing for now
                },
            }
        }
        self.pool().close().await;
        debug!("Closed Database Connection ..");
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct LoginToken {
    pub account_id: u32,
    pub realm_id: u32,
}

#[derive(Clone, Debug)]
pub struct CreationToken {
    pub account_id: u32,
    pub realm_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeneratedLoginToken {
    pub token: u64,
}

/// Listens for the server shutdown signal.
///
/// Shutdown is signalled using a `broadcast::Receiver`. Only a single value is
/// ever sent. Once a value has been sent via the broadcast channel, the server
/// should shutdown.
///
/// The `Shutdown` struct listens for the signal and tracks that the signal has
/// been received. Callers may query for whether the shutdown signal has been
/// received or not.
#[derive(Debug)]
pub struct Shutdown {
    /// `true` if the shutdown signal has been received
    shutdown: bool,

    /// The receive half of the channel used to listen for shutdown.
    notify: broadcast::Receiver<()>,
}

impl Shutdown {
    /// Create a new `Shutdown` backed by the given `broadcast::Receiver`.
    pub fn new(notify: broadcast::Receiver<()>) -> Shutdown {
        Shutdown {
            shutdown: false,
            notify,
        }
    }

    /// Receive the shutdown notice, waiting if necessary.
    pub async fn recv(&mut self) {
        // If the shutdown signal has already been received, then return
        // immediately.
        if self.shutdown {
            return;
        }

        // Cannot receive a "lag error" as only one value is ever sent.
        let _ = self.notify.recv().await;

        // Remember that the signal has been received.
        self.shutdown = true;
    }
}
