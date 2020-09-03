use crate::{
    db,
    world::{Character, Map, Screen, Tile},
    Error,
};
use dashmap::DashMap;
use once_cell::sync::OnceCell;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::{ops::Deref, sync::Arc};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        RwLock,
    },
    task,
};
use tracing::debug;

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
        debug!("Loaded #{} Map From Database", maps.len());
        for map in maps {
            let map = Map::new(map);
            self.maps.insert(map.id(), map);
        }
        Ok(())
    }
}

#[derive(Debug)]
enum StateEvent {
    Map(Map),
    Character(Character),
    Tile(Tile),
    Screen(Screen),
}

#[derive(Debug, Clone)]
pub struct ActorState {
    /// the inner state.
    inner: InnerActorState,
    /// to dispatch events.
    tx: Sender<StateEvent>,
}

impl ActorState {
    pub async fn set_map(&self, map: Map) -> Result<(), Error> {
        self.tx.clone().send(StateEvent::Map(map)).await?;
        task::yield_now().await;
        Ok(())
    }

    pub async fn set_character(
        &self,
        character: Character,
    ) -> Result<(), Error> {
        self.tx
            .clone()
            .send(StateEvent::Character(character))
            .await?;
        task::yield_now().await;
        Ok(())
    }

    pub async fn set_tile(&self, tile: Tile) -> Result<(), Error> {
        self.tx.clone().send(StateEvent::Tile(tile)).await?;
        task::yield_now().await;
        Ok(())
    }

    pub async fn set_screen(&self, screen: Screen) -> Result<(), Error> {
        self.tx.clone().send(StateEvent::Screen(screen)).await?;
        task::yield_now().await;
        Ok(())
    }

    pub async fn map(&self) -> Result<Map, Error> {
        let map = self.inner.map.read().await;
        let map = map.deref().clone();
        Ok(map)
    }

    pub async fn character(&self) -> Result<Character, Error> {
        let character = self.inner.character.read().await;
        let character = character.deref().clone();
        Ok(character)
    }

    pub async fn tile(&self) -> Result<Tile, Error> {
        let tile = self.inner.tile.read().await;
        let tile = *tile.deref();
        Ok(tile)
    }

    pub async fn screen(&self) -> Result<Screen, Error> {
        let screen = self.inner.screen.read().await;
        let screen = screen.deref().clone();
        Ok(screen)
    }
}

impl tq_network::ActorState for ActorState {
    fn init() -> Self {
        let (tx, rx) = mpsc::channel(50);
        let inner = InnerActorState {
            rx: Arc::new(RwLock::new(rx)),
            character: Default::default(),
            map: Default::default(),
            screen: Default::default(),
            tile: Default::default(),
        };
        let state = ActorState {
            tx,
            inner: inner.clone(),
        };
        tokio::spawn(inner.run());
        state
    }
}

#[derive(Debug)]
struct InnerActorState {
    character: Shared<Character>,
    map: Shared<Map>,
    tile: Shared<Tile>,
    screen: Shared<Screen>,
    rx: Shared<Receiver<StateEvent>>,
}

impl InnerActorState {
    async fn run(self) -> Result<(), Error> {
        let mut rx = self.rx.write().await;
        while let Some(event) = rx.recv().await {
            match event {
                StateEvent::Map(map) => {
                    let mut current_map = self.map.write().await;
                    *current_map = map;
                },
                StateEvent::Character(character) => {
                    let mut current_character = self.character.write().await;
                    *current_character = character;
                },
                StateEvent::Tile(tile) => {
                    let mut current_tile = self.tile.write().await;
                    *current_tile = tile;
                },
                StateEvent::Screen(screen) => {
                    let mut current_screen = self.screen.write().await;
                    *current_screen = screen;
                },
            }
        }
        Ok(())
    }
}

impl Clone for InnerActorState {
    fn clone(&self) -> Self {
        Self {
            character: Arc::clone(&self.character),
            map: Arc::clone(&self.map),
            tile: Arc::clone(&self.tile),
            screen: Arc::clone(&self.screen),
            rx: Arc::clone(&self.rx),
        }
    }
}
