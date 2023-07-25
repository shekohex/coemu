use crate::Error;
use async_trait::async_trait;
use bitflags::bitflags;
use primitives::AtomicLocation;
use std::ops::Deref;
use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tq_network::ActorHandle;
mod floor_item;
pub use floor_item::{FloorItem, Item};

bitflags! {
  pub struct EntityTypeFlag: u8 {
    const PLAYER = 1;
    const MONSTER = 2;
    const NPC = 3;
    const TERRAIN_NPC = 4;
  }
}

/// A Base Entity in a game world that has the minimal set of props shared
/// between all of game objects.
#[async_trait]
pub trait BaseEntity {
    fn owner(&self) -> ActorHandle;
    /// The Current Entity Type, used for Casting to other types at runtime if
    /// needed.
    fn entity_type(&self) -> EntityTypeFlag;

    fn is_player(&self) -> bool {
        self.entity_type().contains(EntityTypeFlag::PLAYER)
    }

    fn is_monster(&self) -> bool {
        self.entity_type().contains(EntityTypeFlag::MONSTER)
    }

    fn is_npc(&self) -> bool {
        self.entity_type().contains(EntityTypeFlag::NPC)
    }

    fn is_terrain_npc(&self) -> bool {
        self.entity_type().contains(EntityTypeFlag::TERRAIN_NPC)
    }

    /// This method sends the character's spawn packet to another player. It is
    /// called by the screen system when the players appear in each others'
    /// screens. By default, the actor of the screen change loads the spawn
    /// data for both players.
    async fn send_spawn(&self, to: &ActorHandle) -> Result<(), Error>;
}

#[async_trait]
impl<T: BaseEntity + Send + Sync> BaseEntity for Arc<T> {
    fn owner(&self) -> ActorHandle { self.deref().owner() }

    fn entity_type(&self) -> EntityTypeFlag { self.deref().entity_type() }

    async fn send_spawn(&self, to: &ActorHandle) -> Result<(), Error> {
        let this = &**self;
        this.send_spawn(to).await
    }
}

bitflags! {
  /// These values can be found in `statuseffect.ini` in the `ini` folder of the client.
  /// These values stacked as bitflags to create a composite status effect on the player.
  pub struct Flags: u64 {
    const BLUE_FLASHING_NAME = 1 << 0;
    const POISONED = 1 << 1;
    const REMOVE_MESH = 1 << 2;
    const XP_CIRCLE = 1 << 4;
    const RESTRICT_MOVEMENT = 1 << 5;
    const TEAM_LEADER = 1 << 6;
    const STAR_OF_ACCURACY = 1 << 7;
    const SHIELD = 1 << 8;
    const STIGMA = 1 << 9;
    const DEAD = 1 << 10;
  }
}

impl Default for Flags {
    fn default() -> Self { Flags::empty() }
}

/// A More Advanced Entity Used to be composed with Other Entites Like Player or
/// Monster.
#[derive(Debug, Default)]
pub struct Entity {
    /// Entity Identity in the game world .. Unique over the all game.
    id: u32,
    /// How that entity looks like?
    mesh: AtomicU32,
    /// Could be player name, Monster name .. or anything.
    name: String,
    /// The Current MapID of that entity.
    map_id: AtomicU32,
    /// Current Location (X, Y, Direction)
    location: AtomicLocation,
    /// Set of flags shows the current entity status.
    flags: AtomicU64,
    /// Current Entity Level.
    level: AtomicU16,
    /// What Action this entity is doing right now?
    action: AtomicU16,
    /// Old MapID
    prev_map_id: AtomicU32,
    /// The Old Location .. used in calculations with the new location.
    prev_location: AtomicLocation,
}

impl Entity {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            ..Default::default()
        }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn name(&self) -> String { self.name.clone() }

    pub fn flags(&self) -> Flags {
        Flags::from_bits(self.flags.load(Ordering::Relaxed)).unwrap_or_default()
    }

    pub fn set_flags(&self, flags: Flags) -> &Self {
        self.flags.store(flags.bits(), Ordering::Relaxed);
        self
    }

    pub fn mesh(&self) -> u32 { self.mesh.load(Ordering::Relaxed) }

    pub fn set_mesh(&self, value: u32) -> &Self {
        self.mesh.store(value, Ordering::Relaxed);
        self
    }

    pub fn map_id(&self) -> u32 { self.map_id.load(Ordering::Relaxed) }

    pub fn set_map_id(&self, value: u32) -> &Self {
        let prev_map_id = self.map_id();
        self.prev_map_id.store(prev_map_id, Ordering::Relaxed);
        self.map_id.store(value, Ordering::Relaxed);
        self
    }

    pub fn x(&self) -> u16 { self.location.x.load(Ordering::Relaxed) }

    pub fn set_x(&self, value: u16) -> &Self {
        let x = self.x();
        self.prev_location.x.store(x, Ordering::Relaxed);
        self.location.x.store(value, Ordering::Relaxed);
        self
    }

    pub fn y(&self) -> u16 { self.location.y.load(Ordering::Relaxed) }

    pub fn set_y(&self, value: u16) -> &Self {
        let y = self.y();
        self.prev_location.y.store(y, Ordering::Relaxed);
        self.location.y.store(value, Ordering::Relaxed);
        self
    }

    pub fn direction(&self) -> u8 {
        self.location.direction.load(Ordering::Relaxed)
    }

    pub fn set_direction(&self, value: u8) -> &Self {
        let direction = self.direction();
        self.prev_location
            .direction
            .store(direction, Ordering::Relaxed);
        self.location.direction.store(value, Ordering::Relaxed);
        self
    }

    pub fn level(&self) -> u16 { self.level.load(Ordering::Relaxed) }

    pub fn set_level(&self, value: u16) -> &Self {
        self.level.store(value, Ordering::Relaxed);
        self
    }

    pub fn action(&self) -> u16 { self.action.load(Ordering::Relaxed) }

    pub fn set_action(&self, action: u16) -> &Self {
        self.action.store(action, Ordering::Relaxed);
        self
    }

    pub fn prev_map_id(&self) -> u32 {
        self.prev_map_id.load(Ordering::Relaxed)
    }

    pub fn prev_x(&self) -> u16 { self.prev_location.x.load(Ordering::Relaxed) }

    pub fn prev_y(&self) -> u16 { self.prev_location.y.load(Ordering::Relaxed) }

    pub fn prev_direction(&self) -> u8 {
        self.prev_location.direction.load(Ordering::Relaxed)
    }

    pub fn is_alive(&self) -> bool { !self.flags().contains(Flags::DEAD) }

    pub fn is_dead(&self) -> bool { self.flags().contains(Flags::DEAD) }
}
