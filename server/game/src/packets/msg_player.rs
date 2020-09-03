use crate::world::Character;
use serde::Serialize;
use tq_network::PacketID;

/// This packet is sent to the observing clients on the map
/// server when the actor enters their screen or an acting client observes the
/// character as they enter its screen. The packet contains the player's
/// character spawn information. This class only encapsulates constants related
/// to writing data to the packet buffer. The character class handles writing to
/// the packet as data changes.
#[derive(Debug, Serialize, Clone, PacketID, Default)]
#[packet(id = 1014)]
pub struct MsgPlayer {
    character_id: u32,
    mesh: u32,
    status_flags: u64,
    /// Syndicate
    reserved0: u32,
    germent: u32,
    helment: u32,
    armor: u32,
    right_hand: u32,
    left_hand: u32,
    reserved1: u32,
    health_points: u16,
    level: u16,
    x: u16,
    y: u16,
    hair_style: u16,
    direction: u8,
    action: u8,
    metempsychosis: u16,
    level2: u16,
    reserved2: u32,
    nobility_rank: u32,
    character_id2: u32,
    nobility_position: u32,
    list_count: u8, // 1
    character_name: String,
}

impl From<Character> for MsgPlayer {
    fn from(c: Character) -> Self {
        Self {
            character_id: c.character_id as u32,
            character_id2: c.character_id as u32,
            mesh: c.mesh as u32,
            health_points: c.health_points as u16,
            hair_style: c.hair_style as u16,
            level: c.level as u16,
            level2: c.level as u16,
            x: c.x as u16,
            y: c.y as u16,
            list_count: 1,
            character_name: c.name.clone(),
            ..Default::default()
        }
    }
}
