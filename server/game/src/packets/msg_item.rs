use super::{MsgTalk, TalkChannel};
use crate::state::State;
use crate::ActorState;
use async_trait::async_trait;
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};

/// Enumeration type for defining item actions that may be requested by the
/// user, or given to by the server. Allows for action handling as a packet
/// subtype. Enums should be named by the action they provide to a system in the
/// context of the player item.
#[derive(Default, Debug, FromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u32)]
enum ItemActionType {
    #[default]
    Unknown,
    Buy = 1,
    Sell = 2,
    Drop = 3,
    Use = 4,
    Equip = 5,
    Unequip = 6,
    SplitItem = 7,
    CombineItem = 8,
    QueryMoneySaved = 9,
    SaveMoney = 10,
    DrawMoney = 11,
    DropMoney = 12,
    SpendMoney = 13,
    Repair = 14,
    RepairAll = 15,
    Ident = 16,
    Durability = 17,
    DropEquipement = 18,
    Improve = 19,
    UpLevel = 20,
    BoothQuery = 21,
    BoothAdd = 22,
    BoothDel = 23,
    BoothBuy = 24,
    SynchroAmount = 25,
    Fireworks = 26,
    Ping = 27,
    Enchant = 28,
    BoothAddCPs = 29,
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

    async fn process(&self, _state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        let action = self.action_type.into();
        match action {
            ItemActionType::Ping => {
                // a bit hacky, just testing it out.
                // what if we missed with the client timestamp?
                // does this yield a negative value? let's find out.
                // lets add 30ms from the client timestamp, so when
                // the client receives the packet, it can calculate
                // the round trip time.
                let msg = MsgItem {
                    character_id: self.character_id,
                    param0: self.param0,
                    action_type: self.action_type,
                    client_timestamp: self.client_timestamp + 30,
                    param1: self.param1,
                };
                // LMFAO, this is so bad. it actually made the ping appear
                // negative. I'm not sure if this is a bug in the client
                // or if it's a bug in the server. I'm going to remove this
                // later, but I'm going to leave it here for now.
                actor.send(msg).await?;
            },
            _ => {
                actor.send(self.clone()).await?;
                let p = MsgTalk::from_system(
                    self.character_id,
                    TalkChannel::Service,
                    format!("Missing Item Action Type {:?}", action),
                );
                tracing::warn!(
                    ?action,
                    param0 = %self.param0,
                    param1 = %self.param1,
                    action_id = self.action_type,
                    "Missing Item Action Type",
                );
                actor.send(p).await?;
            },
        }
        Ok(())
    }
}
