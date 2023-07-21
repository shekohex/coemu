use crate::entities::{BaseEntity, Entity, EntityTypeFlag};
use crate::packets::{ActionType, MsgAction, MsgPlayer, MsgTalk, TalkChannel};
use crate::utils::LoHi;
use crate::{constants, db, ActorState, Error};
use std::ops::Deref;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use tq_network::{Actor, IntoErrorPacket};

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
        state: &crate::State,
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
        if let Some(new_map) = state.maps().read().await.get(&map_id) {
            new_map.insert_character(self.clone()).await?;
            let tile = new_map.tile(x, y).await.ok_or_else(|| {
                MsgTalk::from_system(
                    self.id(),
                    TalkChannel::TopLeft,
                    String::from("Invalid Location"),
                )
                .error_packet()
            })?;
            self.set_x(x).set_y(y).set_map_id(map_id);
            self.set_elevation(tile.elevation);
            new_map.update_region_for(self.clone()).await?;
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

    pub async fn save(&self, state: &crate::State) -> Result<(), Error> {
        let e = db::Character {
            character_id: self.inner.character_id,
            account_id: self.inner.account_id,
            realm_id: self.inner.realm_id,
            name: self.name(),
            mesh: self.mesh() as _,
            avatar: self.avatar() as _,
            hair_style: self.hair_style() as _,
            silver: self.silver() as _,
            cps: self.cps() as _,
            current_class: self.current_class() as _,
            previous_class: self.previous_class() as _,
            rebirths: self.rebirths() as _,
            level: self.level() as _,
            experience: self.experience() as _,
            map_id: self.map_id() as _,
            x: self.x() as _,
            y: self.y() as _,
            virtue: self.inner.virtue,
            strength: self.strength() as _,
            agility: self.agility() as _,
            vitality: self.vitality() as _,
            spirit: self.spirit() as _,
            attribute_points: self.attribute_points() as _,
            health_points: self.health_points() as _,
            mana_points: self.mana_points() as _,
            kill_points: self.kill_points() as _,
        };
        e.update(state.pool()).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
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
