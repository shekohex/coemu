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
    syndicate_id: u16,
    /// Unknown
    reserved0: u8,
    syndicate_member_rank: u8,
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
        let msg = Self {
            character_id: c.id(),
            character_id2: c.id(),
            mesh: c.mesh(),
            health_points: c.hp(),
            hair_style: c.hair_style(),
            level: c.level(),
            level2: c.level(),
            x: c.x(),
            y: c.y(),
            list_count: 1,
            character_name: c.name(),
            status_flags: c.flags().bits(),
            direction: c.direction(),
            action: c.action() as u8,
            ..Default::default()
        };
        tracing::debug!("{:?}", msg);
        msg
    }
}
