use crate::{
    db,
    world::{Character, Map, Tile},
    Error,
};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::RwLock;
use tracing::{debug, error, trace};

static STATE: OnceCell<State> = OnceCell::new();

type LoginTokens = Arc<DashMap<u32, (u32, u32)>>;
type CreationTokens = Arc<DashMap<u32, (u32, u32)>>;
type Maps = Arc<DashMap<u32, Map>>;
type Shared<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone)]
pub struct State {
    login_tokens: LoginTokens,
    creation_tokens: CreationTokens,
    maps: Maps,
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
            maps: Arc::new(DashMap::new()),
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
    pub fn pool(&self) -> &PgPool { &self.pool }

    pub fn login_tokens(&self) -> &LoginTokens { &self.login_tokens }

    pub fn creation_tokens(&self) -> &CreationTokens { &self.creation_tokens }

    pub fn maps(&self) -> &Maps { &self.maps }

    /// For Things we should do before sending that we init the state
    async fn post_init() -> Result<(), Error> {
        let state = Self::global()?;
        state.init_maps().await?;
        Ok(())
    }

    /// This method loads the compressed conquer maps from the flat-file
    /// database using the mysql database's maps table. If the map does not
    /// exist, this method will attempt to convert a data map (dmap) file into
    /// a compressed conquer map file (cmap).
    async fn init_maps(&self) -> Result<(), Error> {
        debug!("Loading Maps from Database");
        let maps = db::Map::load_all().await?;
        debug!("Loaded #{} Map", maps.len());
        for map in maps {
            let mut map = Map::new(map);
            let maps = self.maps.clone();
            tokio::spawn(async move {
                if let Err(e) = map.load().await {
                    error!("Error While Loading Map {}: {}", map.id(), e);
                } else {
                    trace!("{} Loaded and Added to the pool", map.id());
                    maps.insert(map.id(), map);
                }
                Result::<_, Error>::Ok(())
            });
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ActorState {
    /// The player's character structure.
    character: Shared<Character>,
    /// The current map the character is playing on.
    map: Shared<Map>,
    /// The current tile the character is standing on.
    tile: Shared<Tile>,
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

    pub async fn map(&self) -> impl Deref<Target = Map> + '_ {
        self.map.read().await
    }

    pub async fn map_mut(&self) -> impl DerefMut<Target = Map> + '_ {
        self.map.write().await
    }

    pub async fn tile(&self) -> impl Deref<Target = Tile> + '_ {
        self.tile.read().await
    }

    pub async fn tile_mut(&self) -> impl DerefMut<Target = Tile> + '_ {
        self.tile.write().await
    }
}
