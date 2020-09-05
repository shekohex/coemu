use super::{MsgTalk, TalkChannel};
use crate::{utils, ActorState};
use async_trait::async_trait;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};
use tracing::warn;
use utils::LoHi;
#[derive(Debug, FromPrimitive)]
#[repr(u16)]
pub enum ActionType {
    #[num_enum(default)]
    Unknown = 0,
    SetLocation = 74,
    SetInventory = 75,
    SetAssociates = 76,
    SetProficiencies = 77,
    SetMagicSpells = 78,
    SetDirection = 79,
    SetAction = 80,
    RequestEntitySpawn = 102,
    SetMapARGB = 104,
    SetLoginComplete = 130,
    RemoveEntity = 135,
    Jump = 137,
}

/// Message containing a general action being performed by the client. Commonly
/// used as a request-response protocol for question and answer like exchanges.
/// For example, walk requests are responded to with an answer as to if the step
/// is legal or not.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PacketID)]
#[packet(id = 1010)]
pub struct MsgAction {
    client_timestamp: u32,
    character_id: u32,
    data1: u32,
    data2: u32,
    details: u16,
    action_type: u16,
}

impl MsgAction {
    pub fn new(
        character_id: u32,
        data1: u32,
        data2: u32,
        details: u16,
        action_type: ActionType,
    ) -> Self {
        Self {
            client_timestamp: utils::current_ts(),
            character_id,
            data1,
            data2,
            details,
            action_type: action_type as u16,
        }
    }
}

#[async_trait]
impl PacketProcess for MsgAction {
    type ActorState = ActorState;
    type Error = crate::Error;

    async fn process(
        &self,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let ty = self.action_type.into();
        match ty {
            ActionType::SetLocation => {
                let mut res = self.clone();
                let character = actor.character().await?;
                res.data1 = character.map_id();
                res.data2 = u32::constract(character.y(), character.x());
                actor.send(res).await?;
            },
            ActionType::SetMapARGB => {
                let mut res = self.clone();
                let character = actor.character().await?;
                res.data1 = 0x00FF_FFFF;
                res.data2 = u32::constract(character.y(), character.x());
                actor.send(res).await?;
            },
            ActionType::RequestEntitySpawn => {
                let mymap = actor.map().await?;
                let other = mymap.characters().get(&self.data1);
                if let Some(other) = other {
                    let msg = super::MsgPlayer::from(other.clone());
                    actor.send(msg).await?;
                }
            },
            _ => {
                let p = MsgTalk::from_system(
                    self.character_id,
                    TalkChannel::Talk,
                    format!("Missing Action Type {:?}", self.action_type),
                );
                actor.send(p).await?;
                let res = self.clone();
                actor.send(res).await?;
                warn!("Missing Action Type {:?}", self.action_type);
            },
        };
        Ok(())
    }
}
