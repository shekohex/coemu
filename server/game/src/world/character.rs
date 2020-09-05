use crate::{
    db,
    entities::{BaseEntity, Entity, EntityTypeFlag},
    packets::{ActionType, MsgAction, MsgPlayer},
    utils::LoHi,
    ActorState, Error,
};
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
#[derive(Debug, Clone, Default)]
pub struct Character {
    inner: Arc<db::Character>,
    entity: Entity,
    owner: Actor<ActorState>,
    elevation: Arc<AtomicU16>,
}

impl Deref for Character {
    type Target = Entity;

    fn deref(&self) -> &Self::Target { &self.entity }
}

impl Character {
    pub fn new(owner: Actor<ActorState>, inner: db::Character) -> Self {
        let entity = Entity::new(inner.character_id as u32, inner.name.clone());
        entity
            .set_x(inner.x as u16)
            .set_y(inner.y as u16)
            .set_map_id(inner.map_id as u32)
            .set_mesh(inner.mesh as u32);
        Self {
            entity,
            owner,
            inner: Arc::new(inner),
            elevation: Default::default(),
        }
    }

    pub fn elevation(&self) -> u16 { self.elevation.load(Ordering::Relaxed) }

    pub fn set_elevation(&self, value: u16) {
        self.elevation.store(value, Ordering::Relaxed);
    }

    pub fn hp(&self) -> u16 { self.inner.health_points as u16 }

    pub fn hair_style(&self) -> u16 { self.inner.hair_style as u16 }

    pub async fn kick_back(&self) -> Result<(), Error> {
        let location = u32::constract(self.y(), self.x());
        let msg = MsgAction::new(
            self.id(),
            self.map_id(),
            location,
            self.direction() as u16,
            ActionType::Teleport,
        );
        self.owner.send(msg).await?;
        Ok(())
    }

    pub async fn exchange_spawn_packets(
        &self,
        observer: impl BaseEntity,
    ) -> Result<(), Error> {
        self.send_spawn(&observer.owner()).await?;
        observer.send_spawn(&self.owner).await?;
        Ok(())
    }
}

#[async_trait]
impl BaseEntity for Character {
    fn owner(&self) -> Actor<ActorState> { self.owner.clone() }

    fn entity_type(&self) -> EntityTypeFlag { EntityTypeFlag::PLAYER }

    fn entity(&self) -> Entity { self.entity.clone() }

    async fn send_spawn(&self, to: &Actor<ActorState>) -> Result<(), Error> {
        let msg = MsgPlayer::from(self.clone());
        to.send(msg).await?;
        Ok(())
    }
}
