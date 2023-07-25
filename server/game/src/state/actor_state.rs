use std::sync::Arc;

use arc_swap::ArcSwapOption;

use crate::systems::Screen;
use crate::world::Character;

#[derive(Debug)]
pub struct ActorState {
    character: ArcSwapOption<Character>,
    screen: ArcSwapOption<Screen>,
}

#[async_trait::async_trait]
impl tq_network::ActorState for ActorState {
    fn init() -> Self {
        ActorState {
            character: Default::default(),
            screen: Default::default(),
        }
    }
}

impl ActorState {
    pub fn set_character(&self, character: Character) {
        self.character.store(Some(Arc::new(character)));
    }

    pub async fn set_screen(&self, screen: Screen) {
        self.screen.store(Some(Arc::new(screen)));
    }

    pub fn character(&self) -> Arc<Character> {
        self.character.load().clone().expect("state is not empty")
    }

    pub fn screen(&self) -> Arc<Screen> {
        self.screen.load().clone().expect("state is not empty")
    }
}
