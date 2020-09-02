use super::ScreenObject;
use crate::{db, ActorState};
use tq_network::Actor;

/// This class encapsulates the game character for a player. The player controls
/// the character as the protagonist of the Conquer Online storyline. The
/// character is the persona of the player who controls it. The persona can be
/// altered using different avatars, hairstyles, and body types. The player also
/// controls the character's professions and abilities.
#[derive(Debug, Clone)]
pub struct Character {
    inner: db::Character,
    owner: Option<Actor<ActorState>>,
    elevation: u16,
}

impl Character {
    pub fn new(owner: Actor<ActorState>, inner: db::Character) -> Self {
        Self {
            inner,
            owner: Some(owner),
            elevation: 0,
        }
    }

    pub fn inner(&self) -> &db::Character { &self.inner }

    pub fn elevation(&self) -> u16 { self.elevation }

    pub fn set_elevation(&mut self, value: u16) { self.elevation = value; }
}

impl Default for Character {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            owner: None,
            elevation: 0,
        }
    }
}

impl ScreenObject for Character {
    fn owner(&self) -> Option<&Actor<ActorState>> { self.owner.as_ref() }

    fn id(&self) -> usize { self.inner.character_id as usize }

    fn x(&self) -> u16 { self.inner.x as u16 }

    fn y(&self) -> u16 { self.inner.y as u16 }

    fn send_spawn(
        &self,
        to: &Actor<ActorState>,
    ) -> Result<(), crate::errors::Error> {
        let _ = to;
        todo!()
    }
}
