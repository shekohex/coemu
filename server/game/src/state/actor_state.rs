use std::sync::Arc;

use tokio::sync::RwLock;
use tq_network::Actor;

use crate::systems::Screen;
use crate::world::{Character, Map};
use crate::Error;

use super::Shared;

#[derive(Debug)]
pub struct ActorState {
    character: Shared<Option<Character>>,
    map: Shared<Option<Map>>,
    screen: Shared<Option<Screen>>,
}

impl Default for ActorState {
    fn default() -> Self {
        Self {
            character: Arc::new(RwLock::new(None)),
            map: Arc::new(RwLock::new(None)),
            screen: Arc::new(RwLock::new(None)),
        }
    }
}
#[async_trait::async_trait]
impl tq_network::ActorState for ActorState {
    fn init() -> Self {
        ActorState {
            character: Default::default(),
            map: Default::default(),
            screen: Default::default(),
        }
    }

    fn empty() -> Self { Self::default() }

    async fn dispose(
        &self,
        actor: &Actor<Self>,
    ) -> Result<(), tq_network::Error> {
        let into = |e: Error| tq_network::Error::Other(e.to_string());
        let mymap = actor.map().await;
        let me = self.character().await;
        mymap.remove_character(me.id()).await.map_err(into)?;
        me.save().await.map_err(into)?;
        let state = super::State::global().map_err(into)?;
        state.characters.write().await.remove(&me.id());
        Ok(())
    }
}

impl ActorState {
    pub async fn set_map(&self, map: Map) {
        let mut lock = self.map.write().await;
        *lock = Some(map);
    }

    pub async fn set_character(&self, character: Character) {
        let mut lock = self.character.write().await;
        *lock = Some(character);
    }

    pub async fn set_screen(&self, screen: Screen) {
        let mut lock = self.screen.write().await;
        *lock = Some(screen);
    }

    pub async fn map(&self) -> Map {
        self.map.read().await.clone().expect("state is not empty")
    }

    pub async fn character(&self) -> Character {
        self.character
            .read()
            .await
            .clone()
            .expect("state is not empty")
    }

    pub async fn screen(&self) -> Screen {
        self.screen
            .read()
            .await
            .clone()
            .expect("state is not empty")
    }
}

impl Clone for ActorState {
    fn clone(&self) -> Self {
        Self {
            character: Arc::clone(&self.character),
            map: Arc::clone(&self.map),
            screen: Arc::clone(&self.screen),
        }
    }
}
