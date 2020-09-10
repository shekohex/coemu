use num_enum::FromPrimitive;
use serde::Serialize;
use tq_network::PacketID;

#[derive(Debug, FromPrimitive)]
#[repr(u8)]
pub enum ItemInfoAction {
    #[num_enum(default)]
    None = 0,
    AddItem = 1,
    Trade = 2,
    Update = 3,
    OtherPlayerEquipement = 4,
}

/// This packet is sent server>client to add or update the attributes of a
/// specific item.
#[derive(Debug, Serialize, Clone, PacketID, Default)]
#[packet(id = 1008)]
pub struct MsgItemInfo {
    character_id: u32,
    item_id: u32,
    durability: u16,
    max_durability: u16,
    action: u8,
    ident: u8, // always 0
    position: u8,
    /// Unknown
    reserved0: u8,
    reserved1: u32,
    gem_one: u8,
    gem_two: u8,
    reborn_effect: u8,
    magic: u8,
    plus: u8,
    blees: u8,
    enchant: u8,
    reserved2: u8,
    restrain: u32,
    reserved3: u32,
    reserved4: u32,
}
