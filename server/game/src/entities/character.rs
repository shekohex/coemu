use crate::entities::{Entity, GameEntity};
use crate::packets::{ActionType, MsgAction, MsgMapInfo, MsgPlayer, MsgWeather};
use crate::systems::Screen;
use crate::utils::LoHi;
use crate::Error;
use arc_swap::ArcSwapWeak;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Weak};
use tq_network::ActorHandle;

/// This struct encapsulates the game character for a player. The player
/// controls the character as the protagonist of the Conquer Online storyline.
/// The character is the persona of the player who controls it. The persona can
/// be altered using different avatars, hairstyles, and body types. The player
/// also controls the character's professions and abilities.
#[derive(Debug)]
pub struct Character {
    inner: tq_db::character::Character,
    entity: Entity,
    owner: ActorHandle,
    elevation: AtomicU16,
    screen: ArcSwapWeak<Screen>,
}

impl Character {
    pub fn new(owner: ActorHandle, inner: tq_db::character::Character) -> Self {
        let entity = Entity::from(&inner);
        Self {
            entity,
            owner,
            inner,
            elevation: Default::default(),
            screen: Default::default(),
        }
    }

    pub fn set_screen(&self, screen: Weak<Screen>) {
        self.screen.store(screen);
    }

    pub fn try_screen(&self) -> Result<Arc<Screen>, Error> {
        self.screen.load().upgrade().ok_or(Error::ScreenNotFound)
    }

    #[inline]
    pub fn owner(&self) -> ActorHandle {
        self.owner.clone()
    }

    #[inline]
    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.entity.id()
    }

    pub fn elevation(&self) -> u16 {
        self.elevation.load(Ordering::Relaxed)
    }

    pub fn set_elevation(&self, value: u16) {
        self.elevation.store(value, Ordering::Relaxed);
    }

    pub fn hair_style(&self) -> u16 {
        self.inner.hair_style as u16
    }

    pub fn avatar(&self) -> u16 {
        self.inner.avatar as u16
    }

    pub fn silver(&self) -> u64 {
        self.inner.silver as u64
    }

    pub fn cps(&self) -> u64 {
        self.inner.cps as u64
    }

    pub fn experience(&self) -> u64 {
        self.inner.experience as u64
    }

    pub fn strength(&self) -> u16 {
        self.inner.strength as u16
    }

    pub fn agility(&self) -> u16 {
        self.inner.agility as u16
    }

    pub fn vitality(&self) -> u16 {
        self.inner.vitality as u16
    }

    pub fn spirit(&self) -> u16 {
        self.inner.spirit as u16
    }

    pub fn attribute_points(&self) -> u16 {
        self.inner.attribute_points as u16
    }

    pub fn health_points(&self) -> u16 {
        self.inner.health_points as u16
    }

    pub fn mana_points(&self) -> u16 {
        self.inner.mana_points as u16
    }

    pub fn kill_points(&self) -> u16 {
        self.inner.kill_points as u16
    }

    pub fn current_class(&self) -> u8 {
        self.inner.current_class as u8
    }

    pub fn previous_class(&self) -> u8 {
        self.inner.previous_class as u8
    }

    pub fn rebirths(&self) -> u8 {
        self.inner.rebirths as u8
    }

    pub async fn kick_back(&self) -> Result<(), Error> {
        let location = self.entity.location();
        let xy = u32::constract(location.y, location.x);
        let msg = MsgAction::new(
            self.entity.id(),
            self.entity.map_id(),
            xy,
            location.direction as u16,
            ActionType::Teleport,
        );
        self.owner.send(msg).await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, state), fields(me = self.entity.id()))]
    pub async fn teleport(&self, state: &crate::State, map_id: u32, (x, y): (u16, u16)) -> Result<(), Error> {
        let mut location = self.entity.location();
        let xy = u32::constract(y, x);
        let msg = MsgAction::new(
            self.entity.id(),
            map_id,
            xy,
            location.direction as u16,
            ActionType::Teleport,
        );
        let new_map = state.try_map(map_id)?;
        new_map.load().await?;
        let tile = new_map.tile(x, y).ok_or(Error::TileNotFound(x, y))?;
        // remove from old map
        if let Ok(old_map) = state.try_map(self.entity.map_id()) {
            old_map.remove_entity_by_id_and_location(self.id(), self.entity().location())?;
            self.try_screen()?.remove_from_observers().await?;
        }
        location.x = x;
        location.y = y;
        self.entity.set_location(location).set_map_id(map_id);
        self.set_elevation(tile.elevation);
        self.owner.send(msg).await?;
        self.owner.send(MsgWeather::new(new_map.weather())).await?;
        self.owner.send(MsgMapInfo::from_map(new_map)).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(me = self.entity.id()))]
    pub async fn exchange_spawn_packets<E: AsRef<GameEntity>>(&self, observer: &E) -> Result<(), Error> {
        match observer.as_ref() {
            GameEntity::Character(c) => {
                self.send_spawn(&c.owner()).await?;
                c.send_spawn(&self.owner).await?;
            },
            _ => {
                // We only exchange spawn packets with characters
            },
        }
        Ok(())
    }

    #[tracing::instrument(skip(self, state), fields(me = self.entity.id()))]
    pub async fn save(&self, state: &crate::State) -> Result<(), Error> {
        let location = self.entity.location();
        let e = tq_db::character::Character {
            character_id: self.inner.character_id,
            account_id: self.inner.account_id,
            realm_id: self.inner.realm_id,
            name: self.entity.name().to_string(),
            mesh: self.entity.mesh() as _,
            avatar: self.avatar() as _,
            hair_style: self.hair_style() as _,
            silver: self.silver() as _,
            cps: self.cps() as _,
            current_class: self.current_class() as _,
            previous_class: self.previous_class() as _,
            rebirths: self.rebirths() as _,
            level: self.entity.level() as _,
            experience: self.experience() as _,
            map_id: self.entity.map_id() as _,
            x: location.x as _,
            y: location.y as _,
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

    #[tracing::instrument(skip(self, to), fields(me = self.entity.id()))]
    pub(super) async fn send_spawn(&self, to: &ActorHandle) -> Result<(), Error> {
        let msg = MsgPlayer::from(self);
        to.send(msg).await?;
        tracing::trace!("Sent Spawn");
        Ok(())
    }
}
