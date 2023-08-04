use crate::entities::Character;
use serde::{Deserialize, Serialize};
use tq_network::PacketID;

/// This packet is sent to the observing clients on the map
/// server when the actor enters their screen or an acting client observes the
/// character as they enter its screen. The packet contains the player's
/// character spawn information. This class only encapsulates constants related
/// to writing data to the packet buffer. The character class handles writing to
/// the packet as data changes.
#[derive(Debug, Serialize, Deserialize, Clone, PacketID, Default)]
#[packet(id = 1014)]
pub struct MsgPlayer {
    pub character_id: i32,
    mesh: i32,
    status_flags: i64,
    syndicate_id: i16,
    /// Unknown
    reserved0: u8,
    syndicate_member_rank: u8,
    germent: i32,
    helment: i32,
    armor: i32,
    right_hand: i32,
    left_hand: i32,
    reserved1: i32,
    health_points: u16,
    level: i16,
    pub x: u16,
    pub y: u16,
    hair_style: i16,
    direction: u8,
    action: u8,
    metempsychosis: i16,
    level2: i16,
    reserved2: i32,
    nobility_rank: i32,
    character_id2: i32,
    nobility_position: i32,
    list_count: u8,
    pub character_name: String,
}

impl From<&Character> for MsgPlayer {
    fn from(c: &Character) -> Self {
        let loc = c.entity().location();
        Self {
            character_id: c.id() as i32,
            character_id2: c.id() as i32,
            mesh: (c.entity().mesh() + (c.avatar() as u32 * 10_000)) as i32,
            health_points: c.entity().hp().current(),
            hair_style: c.hair_style() as i16,
            level: c.entity().level() as i16,
            level2: c.entity().level() as i16,
            x: loc.x,
            y: loc.y,
            direction: loc.direction,
            list_count: 1,
            character_name: c.entity().name().to_owned(),
            status_flags: c.entity().flags().bits() as i64,
            action: c.entity().action() as u8,
            ..Default::default()
        }
    }
}
