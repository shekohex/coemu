use crate::db;
use network::PacketID;
use serde::Serialize;

/// Message defining character information, used to initialize the client
/// interface and game state. Character information is loaded from the game
/// database on login if a character exists.
#[derive(Debug, Serialize, PacketID)]
#[packet(id = 1006)]
pub struct MsgUserInfo {
    character_id: u32,
    mesh: u32,
    hair_style: u16,
    silver: u32,
    cps: u32,
    experience: u64,
    reserved0: u64,
    reserved1: u64,
    strength: u16,
    agility: u16,
    vitality: u16,
    spirit: u16,
    attribute_points: u16,
    health_points: u16,
    mana_points: u16,
    kill_points: u16,
    level: u8,
    current_class: u8,
    previous_class: u8,
    rebirths: u8,
    show_name: bool,
    list_count: u8,
    character_name: String,
    spouse: String,
}

impl Default for MsgUserInfo {
    fn default() -> Self {
        Self {
            character_id: 1,
            mesh: 1003 + 10000,
            hair_style: (3 * 100) + 11,
            silver: 100,
            cps: 0,
            experience: 0,
            reserved0: 0,
            reserved1: 0,
            strength: 4,
            agility: 6,
            vitality: 12,
            spirit: 6,
            attribute_points: 0,
            health_points: 318,
            mana_points: 0,
            kill_points: 0,
            level: 1,
            current_class: 10,
            previous_class: 0,
            rebirths: 0,
            show_name: true,
            list_count: 2,
            character_name: "Test".into(),
            spouse: "None".to_string(),
        }
    }
}

impl From<db::Character> for MsgUserInfo {
    fn from(c: db::Character) -> Self {
        Self {
            character_id: c.character_id as u32,
            mesh: c.mesh as u32,
            hair_style: c.hair_style as u16,
            silver: c.silver as u32,
            cps: c.cps as u32,
            experience: c.experience as u64,
            reserved0: 0,
            reserved1: 0,
            strength: c.strength as u16,
            agility: c.agility as u16,
            vitality: c.vitality as u16,
            spirit: c.spirit as u16,
            attribute_points: c.attribute_points as u16,
            health_points: c.health_points as u16,
            mana_points: c.mana_points as u16,
            kill_points: c.kill_points as u16,
            level: c.level as u8,
            current_class: c.current_class as u8,
            previous_class: c.previous_class as u8,
            rebirths: c.rebirths as u8,
            show_name: true,
            list_count: 2,
            character_name: c.name,
            spouse: "None".to_owned(),
        }
    }
}
