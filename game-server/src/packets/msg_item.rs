use super::{MsgTalk, TalkChannel};
use async_trait::async_trait;
use network::{Actor, PacketID, PacketProcess};
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Enumeration type for defining item actions that may be requested by the
/// user, or given to by the server. Allows for action handling as a packet
/// subtype. Enums should be named by the action they provide to a system in the
/// context of the player item.
#[derive(Debug)]
enum ItemActionType {
    Ping,
    Unknown(u32),
}

impl From<ItemActionType> for u32 {
    fn from(val: ItemActionType) -> u32 {
        match val {
            ItemActionType::Ping => 27,
            ItemActionType::Unknown(val) => val,
        }
    }
}

impl From<u32> for ItemActionType {
    fn from(val: u32) -> ItemActionType {
        match val {
            27 => ItemActionType::Ping,
            val => ItemActionType::Unknown(val),
        }
    }
}
/// Message containing an item action command. Item actions are usually
/// performed to manage player equipment, inventory, money, or item shop
/// purchases and sales. It is serves a second purpose for measuring client
/// ping.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct MsgItem {
    character_id: u32,
    param0: u32,
    action_type: u32,
    client_timestamp: u32,
    param1: u32,
}

impl PacketID for MsgItem {
    fn id(&self) -> u16 { super::PacketType::MsgItem.into() }
}

#[async_trait]
impl PacketProcess for MsgItem {
    type Error = crate::Error;

    async fn process(&self, actor: &Actor) -> Result<(), Self::Error> {
        let action = self.action_type.into();
        match action {
            ItemActionType::Ping => {
                actor.send(self.clone()).await?;
            },
            ItemActionType::Unknown(ty) => {
                actor.send(self.clone()).await?;
                let p = MsgTalk::from_system(
                    self.character_id,
                    TalkChannel::Service,
                    format!("Missing Item Action Type {:?}", ty),
                );
                warn!("Missing Item Action Type {:?}", ty);
                actor.send(p).await?;
            },
        }
        Ok(())
    }
}
