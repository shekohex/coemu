use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU64};

use atomic::{Atomic, Ordering};
use primitives::{Gauge, Location};

use crate::constants;

bitflags::bitflags! {
  /// These values can be found in `statuseffect.ini` in the `ini` folder of the client.
  /// These values stacked as bitflags to create a composite status effect on the player.
  #[repr(transparent)]
  #[derive(Copy, Clone)]
  pub struct Flags: u64 {
    const NONE = 0;
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
    const FADE_OUT = 1 << 11;
    const AZURE_SHIELD = 1 << 12;
    const RED_NAME = 1 << 14;
    const BLACK_NAME = 1 << 15;
    const SUPERMAN = 1 << 18;
    const THIRD_METEMPSYCHOSIS = 1 << 19;
    const FORTH_METEMPSYCHOSIS = 1 << 20;
    const FIFTH_METEMPSYCHOSIS = 1 << 21;
    const INVISIBILITY = 1 << 22;
    const CYCLONE = 1 << 23;
    const SIXTH_METEMPSYCHOSIS = 1 << 24;
    const SEVENTH_METEMPSYCHOSIS = 1 << 25;
    const EIGHTH_METEMPSYCHOSIS = 1 << 26;
    const FLYING = 1 << 27;
    const NINTH_METEMPSYCHOSIS = 1 << 28;
    const TENTH_METEMPSYCHOSIS = 1 << 29;
    const CASTING_PRAY = 1 << 30;
    const PRAYING = 1 << 31;
  }
}

/// A More Advanced Entity Used to be composed with Other Entites Like Player or
/// Monster.
#[derive(Debug)]
pub struct Entity {
    // *** Basic Entity Props ***
    /// Entity Identity in the game world .. Unique over the all game.
    id: u32,
    /// How that entity looks like?
    mesh: AtomicU32,
    /// Could be player name, Monster name .. or anything.
    name: String,
    /// The Current MapID of that entity.
    map_id: AtomicU32,
    /// Current Location (X, Y, Direction)
    location: Atomic<Location>,

    // *** Advanced Entity Props ***
    /// Set of flags shows the current entity status.
    flags: AtomicU64,
    /// Current Entity Level.
    level: AtomicU16,
    /// What Action this entity is doing right now?
    action: AtomicU16,
    /// Old MapID
    prev_map_id: AtomicU32,
    /// The Old Location .. used in calculations with the new location.
    prev_location: Atomic<Location>,
    /// Health Points
    hp: Atomic<Gauge>,
}

impl Entity {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_character(&self) -> bool {
        constants::is_character(self.id)
    }

    pub fn is_npc(&self) -> bool {
        constants::is_npc(self.id)
    }

    pub fn is_monster(&self) -> bool {
        constants::is_monster(self.id)
    }

    pub fn is_pet(&self) -> bool {
        constants::is_pet(self.id)
    }

    pub fn is_call_pet(&self) -> bool {
        constants::is_call_pet(self.id)
    }

    pub fn is_terrain_npc(&self) -> bool {
        constants::is_terrain_npc(self.id)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits(self.flags.load(Ordering::Relaxed)).unwrap_or(Flags::NONE)
    }

    pub fn set_flags(&self, flags: Flags) -> &Self {
        self.flags.store(flags.bits(), Ordering::Relaxed);
        self
    }

    pub fn mesh(&self) -> u32 {
        self.mesh.load(Ordering::Relaxed)
    }

    pub fn set_mesh(&self, value: u32) -> &Self {
        self.mesh.store(value, Ordering::Relaxed);
        self
    }

    pub fn map_id(&self) -> u32 {
        self.map_id.load(Ordering::Relaxed)
    }

    pub fn set_map_id(&self, value: u32) -> &Self {
        let prev_map_id = self.map_id();
        self.prev_map_id.store(prev_map_id, Ordering::Relaxed);
        self.map_id.store(value, Ordering::Relaxed);
        self
    }

    pub fn location(&self) -> Location {
        self.location.load(Ordering::Relaxed)
    }

    pub fn set_location(&self, value: Location) -> &Self {
        let prev_location = self.location();
        self.prev_location.store(prev_location, Ordering::Relaxed);
        self.location.store(value, Ordering::Relaxed);
        self
    }

    pub fn level(&self) -> u16 {
        self.level.load(Ordering::Relaxed)
    }

    pub fn set_level(&self, value: u16) -> &Self {
        self.level.store(value, Ordering::Relaxed);
        self
    }

    pub fn action(&self) -> u16 {
        self.action.load(Ordering::Relaxed)
    }

    pub fn set_action(&self, action: u16) -> &Self {
        self.action.store(action, Ordering::Relaxed);
        self
    }

    pub fn prev_map_id(&self) -> u32 {
        self.prev_map_id.load(Ordering::Relaxed)
    }

    pub fn prev_location(&self) -> Location {
        self.prev_location.load(Ordering::Relaxed)
    }

    pub fn hp(&self) -> Gauge {
        self.hp.load(Ordering::Relaxed)
    }

    pub fn is_alive(&self) -> bool {
        !self.flags().contains(Flags::DEAD)
    }

    pub fn is_dead(&self) -> bool {
        self.flags().contains(Flags::DEAD)
    }
}

impl From<&tq_db::character::Character> for Entity {
    fn from(v: &tq_db::character::Character) -> Self {
        // TODO: handle more flags.
        let flags = {
            let f = Flags::NONE;
            match v.kill_points as u16 {
                30..=99 => f | Flags::RED_NAME,
                100.. => f | Flags::BLACK_NAME,
                _ => f,
            };
            f
        };
        Self {
            id: (v.character_id as u32) + constants::CHARACTER_ID_MIN,
            mesh: AtomicU32::new(v.mesh as _),
            name: v.name.clone(),
            map_id: AtomicU32::new(v.map_id as _),
            location: Atomic::new(Location::new(v.x as _, v.y as _, 0)),
            flags: AtomicU64::new(flags.bits()),
            level: AtomicU16::new(v.level as _),
            action: AtomicU16::new(100),
            prev_map_id: AtomicU32::new(v.map_id as _),
            prev_location: Atomic::new(Location::default()),
            hp: Atomic::new(Gauge {
                current: v.health_points as _,
                // TODO: handle max hp.
                max: v.health_points as _,
            }),
        }
    }
}

impl From<&tq_db::npc::Npc> for Entity {
    fn from(v: &tq_db::npc::Npc) -> Self {
        Self {
            id: (v.id as u32),
            mesh: AtomicU32::new(v.look as _),
            name: v.name.clone(),
            map_id: AtomicU32::new(v.map_id as _),
            location: Atomic::new(Location::new(v.x as _, v.y as _, (v.look % 10) as _)),
            flags: AtomicU64::new(Flags::NONE.bits()),
            level: AtomicU16::new(v.level as _),
            action: AtomicU16::new(100),
            prev_map_id: AtomicU32::new(v.map_id as _),
            prev_location: Atomic::new(Location::default()),
            hp: Atomic::new(Gauge::default()),
        }
    }
}
