use crate::Error;
use tq_network::ActorHandle;

mod floor_item;
pub use floor_item::{FloorItem, Item};

mod basic;
pub use basic::Entity;

mod character;
pub use character::Character;

mod npc;
pub use npc::{Npc, NpcBase, NpcKind, NpcSort};

#[derive(Debug)]
pub enum GameEntity {
    Character(Character),
    Npc(Npc),
}

impl From<Character> for GameEntity {
    fn from(v: Character) -> Self {
        Self::Character(v)
    }
}

impl From<Npc> for GameEntity {
    fn from(v: Npc) -> Self {
        Self::Npc(v)
    }
}

impl GameEntity {
    /// Returns the ID of the Game Entity.
    pub fn id(&self) -> u32 {
        match self {
            Self::Character(v) => v.id(),
            Self::Npc(v) => v.id(),
        }
    }

    /// Returns the Owner of the Game Entity.
    ///
    /// Only [`Character`]s have an owner.
    pub fn owner(&self) -> Option<ActorHandle> {
        match self {
            Self::Character(v) => Some(v.owner()),
            Self::Npc(..) => None,
        }
    }

    pub fn basic(&self) -> &Entity {
        match self {
            Self::Character(v) => v.entity(),
            Self::Npc(v) => v.entity(),
        }
    }

    /// This method sends the spawn packet to another entity
    pub async fn send_spawn(&self, to: &Self) -> Result<(), Error> {
        match (self, to) {
            (Self::Character(from), Self::Character(to)) => from.send_spawn(&to.owner()).await,
            (Self::Npc(from), Self::Character(to)) => from.send_spawn(&to.owner()).await,
            _ => todo!("send_spawn for non-character entities"),
        }
    }

    /// Returns `true` if the game entity is [`Character`].
    ///
    /// [`Character`]: GameEntity::Character
    #[must_use]
    pub fn is_character(&self) -> bool {
        matches!(self, Self::Character(..))
    }

    pub fn as_character(&self) -> Option<&Character> {
        if let Self::Character(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the game entity is [`Npc`].
    ///
    /// [`Npc`]: GameEntity::Npc
    #[must_use]
    pub fn is_npc(&self) -> bool {
        matches!(self, Self::Npc(..))
    }

    pub fn as_npc(&self) -> Option<&Npc> {
        if let Self::Npc(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
