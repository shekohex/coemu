use serde::Serialize;
use tq_network::PacketID;

use crate::world::Map;

bitflags::bitflags! {
  #[repr(transparent)]
  #[derive(Default, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
  pub struct MapFlags: u32 {
    const NONE = 0;
    /// No PkPoints, Not Flashing.
    const PK_FIELD = 1 << 0;
    /// No Change Map.
    const CHANGE_MAP_DISABLED = 1 << 1;
    /// Do not save this position, save the previous
    const RECORD_DISABLED = 1 << 2;
    /// No PK.
    const PK_DISABLED = 1 << 3;
    /// Booth enabled.
    const BOOTH_ENABLED = 1 << 4;
    /// Team Disabled.
    const TEAM_DISABLED = 1 << 5;
    /// Teleport Disabled.
    const TELEPORT_DISABLED = 1 << 6;
    /// Syndicate Map.
    const SYNDICATE_MAP = 1 << 7;
    /// Prison Map
    const PRISON_MAP = 1 << 8;
    /// Can't fly.
    const FLY_DISABLED = 1 << 9;
    /// Family Map.
    const FAMILY_MAP = 1 << 10;
    /// Mine Map.
    const MINE_FIELD = 1 << 11;
    /// Free For All Map. (No Newbie Protection)
    const FFA_MAP = 1 << 12;
    /// Blessed reborn map.
    const BLESSED_REBORN_MAP = 1 << 13;
    /// Neobiess Protection.
    const NEWBIE_PROTECTION = 1 << 14;
  }
}

/// This packet is sent from the game server to the game client to set game map
/// rules and details. The packet may be used to set game map rules, such as
/// no-pk and no-jump. It may also be used to create dynamic map copies of
/// static maps, or static copies of static maps. The game identity comes from
/// GameMap.dat. The unique map identity will be the same as the game map
/// identity if the map is static (not a copy).
#[derive(Debug, Serialize, Clone, PacketID)]
#[packet(id = 1110)]
pub struct MsgMapInfo {
    uid: u32,
    map_id: u32,
    flags: MapFlags,
}

impl MsgMapInfo {
    pub fn from_map(map: &Map) -> Self {
        Self {
            uid: map.id(),
            map_id: map.map_id(),
            flags: map.flags(),
        }
    }

    pub fn is_static(&self) -> bool {
        self.uid == self.map_id
    }

    pub fn is_copy(&self) -> bool {
        self.uid != self.map_id
    }
}

impl From<&Map> for MsgMapInfo {
    fn from(map: &Map) -> Self {
        Self::from_map(map)
    }
}
