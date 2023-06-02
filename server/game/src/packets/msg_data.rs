use crate::{ActorState, Error};
use chrono::{Datelike, Timelike};
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};

/// Enumeration type for defining data actions that may used by the client.
#[derive(Debug, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum DataAction {
    #[default]
    SetServerTime = 0,
    SetMountMovePoint = 2,
    AntiCheatAnswerMsgTypeCount = 3,
    AntiCheatAskMsgTypeCount = 4,
}

/// Message containing the current date and time. This is sent to the client
/// to synchronize the client's clock with the server's clock.
#[derive(Debug, Default, Deserialize, Serialize, PacketID)]
#[packet(id = 1033)]
pub struct MsgData {
    action: u32,
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: i32,
}

impl MsgData {
    pub fn now() -> Self {
        let now = chrono::Utc::now();
        Self {
            action: DataAction::SetServerTime.into(),
            year: now.year() - 1900,
            month: (now.month() - 1) as i32,
            day: now.day() as i32,
            hour: now.hour() as i32,
            minute: now.minute() as i32,
            second: now.second() as i32,
        }
    }
}

#[async_trait::async_trait]
impl PacketProcess for MsgData {
    type ActorState = ActorState;
    type Error = Error;

    async fn process(
        &self,
        _actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
