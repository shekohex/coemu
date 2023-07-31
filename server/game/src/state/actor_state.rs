use std::sync::{Arc, Weak};

use arc_swap::ArcSwapOption;

use crate::systems::Screen;
use crate::world::Character;
use crate::Error;

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
    pub fn update(&self, character: Character, screen: Screen) {
        let character = Arc::new(character);
        let screen = Arc::new(screen);
        // We use Weak references to avoid a circular reference between the
        // character and the screen. The screen needs to know about the
        // character, and the character needs to know about the screen. However,
        // if the character holds a strong reference to the screen, then the
        // screen will never be dropped. and the other way around.
        // hence, we use a weak reference to the screen and the character. This
        // if the screen is dropped, then the character will be dropped as well.
        character.set_screen(Arc::downgrade(&screen));
        screen.set_character(Arc::downgrade(&character));
        self.character.store(Some(character));
        self.screen.store(Some(screen));
    }

    pub fn character(&self) -> Arc<Character> {
        self.character.load().clone().expect("state is not empty")
    }

    pub fn character_weak(&self) -> Weak<Character> {
        let character = self.character.load().clone();
        match character {
            Some(character) => Arc::downgrade(&character),
            None => Weak::new(),
        }
    }

    pub fn try_character(&self) -> Result<Arc<Character>, Error> {
        self.character
            .load()
            .clone()
            .ok_or(Error::CharacterNotFound)
    }

    pub fn try_character_weak(&self) -> Result<Weak<Character>, Error> {
        let character = self.character.load().clone();
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
