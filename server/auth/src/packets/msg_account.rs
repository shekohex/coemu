use super::{MsgConnectEx, MsgTransfer};
use crate::packets::RejectionCode;
use crate::state::State;
use crate::Error;
use async_trait::async_trait;
use serde::Deserialize;
use tq_db::account::Account;
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
        let maybe_accont =
            Account::auth(pool, &self.username, &self.password).await;
        let account = match maybe_accont {
            Ok(account) => account,
            Err(e) => {
                let res = match e {
                    tq_db::Error::InvalidPassword
                    | tq_db::Error::AccountNotFound => {
                        RejectionCode::InvalidPassword.packet()
                    },
                    _ => {
                        tracing::error!("Error authenticating account: {e}");
                        RejectionCode::TryAgainLater.packet()
                    },
                };
                actor.send(res).await?;
                return Ok(());
            },
        };
        actor.set_id(account.account_id as usize);
        let res = match MsgTransfer::handle(state, actor, &self.realm).await {
            Ok(res) => res,
            _ => {
                tracing::warn!(
                    account_id = account.account_id,
                    "Failed to transfer account"
                );
                return Ok(());
            },
        };
        let res = MsgConnectEx::forword_connection(res);
        actor.send(res).await?;
        Ok(())
    }
}
