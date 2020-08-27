use super::{MsgTalk, TalkChannel};
use crate::utils;
use async_trait::async_trait;
use network::{Actor, PacketID, PacketProcess};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug)]
enum ActionType {
    Unknown = 0,
    SetLocation = 74,
    SetInventory = 75,
    SetAssociates = 76,
    SetProficiencies = 77,
    SetMagicSpells = 78,
    SetDirection = 79,
    SetAction = 80,
    SetMapARGB = 104,
}

impl From<u16> for ActionType {
    fn from(val: u16) -> ActionType {
        match val {
            74 => ActionType::SetLocation,
            75 => ActionType::SetInventory,
            76 => ActionType::SetAssociates,
            77 => ActionType::SetProficiencies,
            78 => ActionType::SetMagicSpells,
            79 => ActionType::SetDirection,
            80 => ActionType::SetAction,
            104 => ActionType::SetMapARGB,
            _ => ActionType::Unknown,
        }
    }
}

/// Message containing a general action being performed by the client. Commonly
/// used as a request-response protocol for question and answer like exchanges.
/// For example, walk requests are responded to with an answer as to if the step
/// is legal or not.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct MsgAction {
    client_timestamp: u32,
    character_id: u32,
    param0: u32,
    param1: u16,
    param2: u16,
    param3: u16,
    action_type: u16,
}

impl PacketID for MsgAction {
    fn id(&self) -> u16 { super::PacketType::MsgAction.into() }
}

#[async_trait]
impl PacketProcess for MsgAction {
    type Error = crate::Error;

    async fn process(&self, actor: &Actor) -> Result<(), Self::Error> {
        let ty = self.action_type.into();
        match ty {
            ActionType::SetLocation => {
                let mut res = self.clone();
                res.client_timestamp = utils::current_tick_ms();
                res.param0 = 1002;
                res.param1 = 430;
                res.param2 = 380;
                actor.send(res).await?;
            },
            ActionType::SetMapARGB => {
                let mut res = self.clone();
                res.client_timestamp = utils::current_tick_ms();
                res.param0 = 0x00FF_FFFF;
                res.param1 = 430;
                res.param2 = 380;
                actor.send(res).await?;
            },
            _ => {
                let p = MsgTalk::from_system(
                    self.character_id,
                    TalkChannel::Talk,
                    format!("Missing Action Type {:?}", ty),
                );
                actor.send(p).await?;
                let mut res = self.clone();
                res.client_timestamp = utils::current_tick_ms();
                actor.send(res).await?;
                warn!("Missing Action Type {:?}", ty);
            },
        };
        Ok(())
    }
}
