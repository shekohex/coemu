use crate::{Error, State};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};

/// Defines account parameters to be transferred from the account server to the
/// game server. Account information is supplied from the account database, and
/// used on the game server to transfer authentication and authority level.  
#[derive(Clone, Debug, Deserialize, Serialize, PacketID)]
#[packet(id = 4001)]
pub struct MsgTransfer {
    account_id: u32,
    realm_id: u32,
    #[serde(skip_deserializing)]
    token: u32,
    #[serde(skip_deserializing)]
    code: u32,
}

#[async_trait]
impl PacketProcess for MsgTransfer {
    type ActorState = ();
    type Error = Error;

    async fn process(
        &self,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let generated = State::global()?
            .token_store()
            .generate_login_token(self.account_id, self.realm_id)
            .await?;
        let mut msg = self.clone();
        msg.token = generated.token;
        msg.code = generated.code;
        actor.send(msg).await?;
        actor.shutdown().await?;
        Ok(())
    }
}
