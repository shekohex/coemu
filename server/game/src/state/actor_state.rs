use std::sync::{Arc, Weak};

use arc_swap::ArcSwapOption;

use crate::entities::{Character, GameEntity};
use crate::systems::Screen;
use crate::Error;

#[derive(Debug)]
pub struct ActorState {
    entity: ArcSwapOption<GameEntity>,
    screen: ArcSwapOption<Screen>,
}

#[async_trait::async_trait]
impl tq_network::ActorState for ActorState {
    fn init() -> Self {
        ActorState {
            entity: Default::default(),
            screen: Default::default(),
        }
    }
}

impl ActorState {
    pub fn update(&self, character: Character, screen: Screen) {
        let screen = Arc::new(screen);
        character.set_screen(Arc::downgrade(&screen));
        let character = Arc::new(GameEntity::Character(character));
        // We use Weak references to avoid a circular reference between the
        // character and the screen. The screen needs to know about the
        // character, and the character needs to know about the screen. However,
        // if the character holds a strong reference to the screen, then the
        // screen will never be dropped. and the other way around.
        // hence, we use a weak reference to the screen and the character. This
        // if the screen is dropped, then the character will be dropped as well.
        screen.set_character(Arc::downgrade(&character));
        self.entity.store(Some(character));
        self.screen.store(Some(screen));
    }

    pub fn entity(&self) -> Arc<GameEntity> {
        self.entity.load().clone().expect("state is not empty")
    }

    pub fn entity_weak(&self) -> Weak<GameEntity> {
        let e = self.entity.load().clone();
        match e {
            Some(e) => Arc::downgrade(&e),
            None => Weak::new(),
        }
    }

    pub fn try_entity(&self) -> Result<Arc<GameEntity>, Error> {
        self.entity.load().clone().ok_or(Error::CharacterNotFound)
    }

    pub fn try_entity_weak(&self) -> Result<Weak<GameEntity>, Error> {
        let character = self.entity.load().clone();
        match character {
            Some(character) => Ok(Arc::downgrade(&character)),
            None => Err(Error::CharacterNotFound),
        }
    }

    pub fn screen(&self) -> Arc<Screen> {
        self.screen.load().clone().expect("state is not empty")
    }

    pub fn screen_weak(&self) -> Weak<Screen> {
        let screen = self.screen.load().clone();
        match screen {
            Some(screen) => Arc::downgrade(&screen),
            None => Weak::new(),
        }
    }

    pub fn try_screen(&self) -> Result<Arc<Screen>, Error> {
        self.screen.load().clone().ok_or(Error::ScreenNotFound)
    }

    pub fn try_screen_weak(&self) -> Result<Weak<Screen>, Error> {
        let screen = self.screen.load().clone();
        match screen {
            Some(screen) => Ok(Arc::downgrade(&screen)),
            None => Err(Error::ScreenNotFound),
        }
    }
}
