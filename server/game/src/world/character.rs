use crate::{
    constants, db,
    entities::{BaseEntity, Entity, EntityTypeFlag},
    packets::{ActionType, MsgAction, MsgPlayer},
    utils::LoHi,
    ActorState, Error, State,
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
        let entity = Entity::new(
            inner.character_id as u32 + constants::CHARACTER_BASE_ID,
            inner.name.clone(),
        );
        entity
            .set_x(inner.x as u16)
            .set_y(inner.y as u16)
            .set_map_id(inner.map_id as u32)
            .set_level(inner.level as u16)
            .set_action(100)
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

    pub fn avatar(&self) -> u16 { self.inner.avatar as u16 }

    pub fn silver(&self) -> u64 { self.inner.silver as u64 }

    pub fn cps(&self) -> u64 { self.inner.cps as u64 }

    pub fn experience(&self) -> u64 { self.inner.experience as u64 }

    pub fn strength(&self) -> u16 { self.inner.strength as u16 }

    pub fn agility(&self) -> u16 { self.inner.agility as u16 }

    pub fn vitality(&self) -> u16 { self.inner.vitality as u16 }

    pub fn spirit(&self) -> u16 { self.inner.spirit as u16 }

    pub fn attribute_points(&self) -> u16 { self.inner.attribute_points as u16 }

    pub fn health_points(&self) -> u16 { self.inner.health_points as u16 }

    pub fn mana_points(&self) -> u16 { self.inner.mana_points as u16 }

    pub fn kill_points(&self) -> u16 { self.inner.kill_points as u16 }

    pub fn current_class(&self) -> u8 { self.inner.current_class as u8 }

    pub fn previous_class(&self) -> u8 { self.inner.previous_class as u8 }

    pub fn rebirths(&self) -> u8 { self.inner.rebirths as u8 }

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

    pub async fn teleport(
        &self,
        map_id: u32,
        (x, y): (u16, u16),
    ) -> Result<(), Error> {
        let location = u32::constract(y, x);
        let msg = MsgAction::new(
            self.id(),
            map_id,
            location,
            self.direction() as u16,
            ActionType::Teleport,
        );
        let state = State::global()?;
        if let Some(new_map) = state.maps().read().await.get(&map_id) {
            new_map.insert_character(self.clone()).await?;
            self.set_x(x).set_y(y).set_map_id(map_id);
            self.owner.send(msg).await?;
        }
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
