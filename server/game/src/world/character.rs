use super::ScreenObject;
use crate::{db, packets::MsgPlayer, ActorState, Error};
use async_trait::async_trait;
use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicU16, Ordering},
        Arc,
    },
};
use tq_network::Actor;

/// This struct encapsulates the game character for a player. The player
/// controls the character as the protagonist of the Conquer Online storyline.
/// The character is the persona of the player who controls it. The persona can
/// be altered using different avatars, hairstyles, and body types. The player
/// also controls the character's professions and abilities.
#[derive(Debug, Clone)]
pub struct Character {
    inner: Arc<db::Character>,
    owner: Option<Actor<ActorState>>,
    elevation: Arc<AtomicU16>,
}

impl Deref for Character {
    type Target = db::Character;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl Character {
    pub fn new(owner: Actor<ActorState>, inner: db::Character) -> Self {
        Self {
            inner: Arc::new(inner),
            owner: Some(owner),
            elevation: Default::default(),
        }
    }

    pub fn elevation(&self) -> u16 { self.elevation.load(Ordering::Relaxed) }

    pub fn set_elevation(&self, value: u16) {
        self.elevation.store(value, Ordering::Relaxed);
    }

    pub async fn exchange_spawn_packets(
        &self,
        observer: impl ScreenObject,
    ) -> Result<(), Error> {
        if let (Some(observer_owner), Some(myowner)) =
            (observer.owner(), self.owner())
        {
            self.send_spawn(&observer_owner).await?;
            observer.send_spawn(&myowner).await?;
        }
        Ok(())
    }
}

impl Default for Character {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            elevation: Default::default(),
            owner: None,
        }
    }
}

#[async_trait]
impl ScreenObject for Character {
    fn owner(&self) -> Option<Actor<ActorState>> { self.owner.clone() }

    fn id(&self) -> usize { self.inner.character_id as usize }

    fn x(&self) -> u16 { self.inner.x as u16 }

    fn y(&self) -> u16 { self.inner.y as u16 }

    fn is_charachter(&self) -> bool { true }

    async fn send_spawn(&self, to: &Actor<ActorState>) -> Result<(), Error> {
        let msg = MsgPlayer::from(self.clone());
        to.send(msg).await?;
        Ok(())
    }
}
