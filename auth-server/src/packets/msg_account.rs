use super::{MsgConnectEx, RejectionCode};
use async_trait::async_trait;
use network::{Actor, PacketProcess};
use serde::Deserialize;
use tq_serde::{String16, TQPassword};

#[derive(Debug, Default, Deserialize)]
pub struct MsgAccount {
    pub username: String16,
    pub password: TQPassword,
    pub realm: String16,
    #[serde(skip)]
    pub rejection_code: u32,
    #[serde(skip)]
    pub account_id: i32,
}

#[async_trait]
impl PacketProcess for MsgAccount {
    type Error = crate::Error;

    async fn process(&self, actor: &Actor) -> Result<(), Self::Error> {
        let res = MsgConnectEx::from_code(RejectionCode::InvalidPassword);
        actor.send(res).await?;
        Ok(())
    }
}
