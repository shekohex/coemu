use crate::{
    db,
    systems::Screen,
    world::{Character, Map},
    Error,
};
use async_trait::async_trait;
use once_cell::sync::OnceCell;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::{
    sync::{
        mpsc::{self, Receiver, Sender},
        RwLock,
    },
    task,
};
use tq_network::Actor;
use tracing::debug;

static STATE: OnceCell<State> = OnceCell::new();

type Tokens = Arc<RwLock<HashMap<u32, (u32, u32)>>>;
type Maps = Arc<RwLock<HashMap<u32, Map>>>;
type Characters = Arc<RwLock<HashMap<u32, Character>>>;
type Shared<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone)]
pub struct State {
    login_tokens: Tokens,
    creation_tokens: Tokens,
    characters: Characters,
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
            login_tokens: Default::default(),
            creation_tokens: Default::default(),
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
    pub fn pool(&self) -> &PgPool { &self.pool }

    pub fn login_tokens(&self) -> &Tokens { &self.login_tokens }

    pub fn creation_tokens(&self) -> &Tokens { &self.creation_tokens }

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
        for map in maps {
            let map = Map::new(map);
            let mut maps = self.maps.write().await;
            maps.insert(map.id(), map);
        }
        Ok(())
    }
}

#[derive(Debug)]
enum StateEvent {
    Map(Map),
    Character(Character),
    Screen(Screen),
}

#[derive(Debug, Clone)]
pub struct ActorState {
    /// the inner state.
    inner: Option<InnerActorState>,
    /// to dispatch events.
    tx: Sender<StateEvent>,
}

impl Default for ActorState {
    fn default() -> Self {
        let (tx, _) = mpsc::channel(1);
        Self {
            inner: Default::default(),
            tx,
        }
    }
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

    pub async fn set_screen(&self, screen: Screen) -> Result<(), Error> {
        self.tx.clone().send(StateEvent::Screen(screen)).await?;
        task::yield_now().await;
        Ok(())
    }

    pub async fn map(&self) -> Result<Map, Error> {
        let inner = self.inner();
        let map = inner.map.read().await;
        let map = map.deref().clone();
        Ok(map)
    }

    pub async fn character(&self) -> Result<Character, Error> {
        let inner = self.inner();
        let character = inner.character.read().await;
        let character = character.deref().clone();
        Ok(character)
    }

    pub async fn screen(&self) -> Result<Screen, Error> {
        let inner = self.inner();
        let screen = inner.screen.read().await;
        let screen = screen.deref().clone();
        Ok(screen)
    }

    #[track_caller]
    fn inner(&self) -> InnerActorState { self.inner.clone().unwrap() }
}

#[async_trait]
impl tq_network::ActorState for ActorState {
    fn init() -> Self {
        let (tx, rx) = mpsc::channel(50);
        let inner = InnerActorState {
            rx: Arc::new(RwLock::new(rx)),
            character: Default::default(),
            map: Default::default(),
            screen: Default::default(),
        };
        let state = ActorState {
            tx,
            inner: Some(inner.clone()),
        };
        tokio::spawn(inner.run());
        state
    }

    fn empty() -> Self {
        let (tx, _) = mpsc::channel(1);
        Self { inner: None, tx }
    }

    async fn dispose(
        &self,
        actor: &Actor<Self>,
    ) -> Result<(), tq_network::Error> {
        let into = |e: Error| tq_network::Error::Other(e.to_string());
        let mymap = actor.map().await.map_err(into)?;
        let me = self.character().await.map_err(into)?;
        mymap.remove_character(me.id()).await.map_err(into)?;
        me.save().await.map_err(into)?;
        let state = State::global().map_err(into)?;
        state.characters.write().await.remove(&me.id());
        Ok(())
    }
}

#[derive(Debug)]
struct InnerActorState {
    character: Shared<Character>,
    map: Shared<Map>,
    screen: Shared<Screen>,
    rx: Shared<Receiver<StateEvent>>,
}

impl Default for InnerActorState {
    fn default() -> Self {
        let (_, rx) = mpsc::channel(1);
        Self {
            character: Default::default(),
            map: Default::default(),
            screen: Default::default(),
            rx: Arc::new(RwLock::new(rx)),
        }
    }
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
            screen: Arc::clone(&self.screen),
            rx: Arc::clone(&self.rx),
        }
    }
}
