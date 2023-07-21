use super::{MsgConnectEx, MsgTransfer};
use crate::state::State;
use crate::{db, Error};
use async_trait::async_trait;
use serde::Deserialize;
use tq_network::{Actor, PacketID, PacketProcess};
use tq_serde::{String16, TQPassword};

#[derive(Debug, Deserialize, PacketID)]
#[packet(id = 1051)]
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
    type ActorState = ();
    type Error = Error;
    type State = State;

    async fn process(
        &self,
        state: &Self::State,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let pool = state.pool();
        let account =
            db::Account::auth(pool, &self.username, &self.password).await?;
        actor.set_id(account.account_id as usize);
        let res = MsgTransfer::handle(state, actor, &self.realm).await?;
        let res = MsgConnectEx::forword_connection(res);
        actor.send(res).await?;
        Ok(())
    }
}
