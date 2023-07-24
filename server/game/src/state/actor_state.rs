use std::sync::Arc;

use futures::TryFutureExt;
use tq_network::Actor;

use crate::systems::Screen;
use crate::world::{Character, Map};

use super::Shared;

#[derive(Debug)]
pub struct ActorState {
    character: Shared<Option<Character>>,
    map: Shared<Option<Map>>,
    screen: Shared<Option<Screen>>,
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

    async fn dispose(
        &self,
        actor: &Actor<Self>,
    ) -> Result<(), tq_network::Error> {
        let mymap = actor.map().await;
        let me = self.character().await;
        mymap
            .remove_character(me.id())
            .map_err(|e| tq_network::Error::Other(e.to_string()))
            .await?;
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
