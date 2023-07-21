use super::{MsgTalk, TalkChannel};
use crate::state::State;
use crate::ActorState;
use async_trait::async_trait;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};
use tracing::warn;

/// Enumeration type for defining item actions that may be requested by the
/// user, or given to by the server. Allows for action handling as a packet
/// subtype. Enums should be named by the action they provide to a system in the
/// context of the player item.
#[derive(Debug, FromPrimitive)]
#[repr(u32)]
enum ItemActionType {
    Ping = 27,
    #[num_enum(default)]
    Unknown,
}

/// Message containing an item action command. Item actions are usually
/// performed to manage player equipment, inventory, money, or item shop
/// purchases and sales. It is serves a second purpose for measuring client
/// ping.
#[derive(Debug, Serialize, Deserialize, Clone, PacketID)]
#[packet(id = 1009)]
pub struct MsgItem {
    character_id: u32,
    param0: u32,
    action_type: u32,
    client_timestamp: u32,
    param1: u32,
}

#[async_trait]
impl PacketProcess for MsgItem {
    type ActorState = ActorState;
    type Error = crate::Error;
    type State = State;

    async fn process(
        &self,
        _state: &Self::State,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let action = self.action_type.into();
        match action {
            ItemActionType::Ping => {
                actor.send(self.clone()).await?;
                actor.send(super::MsgData::now()).await?;
            },
            ItemActionType::Unknown => {
                actor.send(self.clone()).await?;
                let p = MsgTalk::from_system(
                    self.character_id,
                    TalkChannel::Service,
                    format!("Missing Item Action Type {:?}", self.action_type),
                );
                warn!("Missing Item Action Type {:?}", self.action_type);
                actor.send(p).await?;
            },
        }
        Ok(())
    }
}
