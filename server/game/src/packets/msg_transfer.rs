use crate::{ActorState, Error, State};
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
    token: u64,
}

#[async_trait]
impl PacketProcess for MsgTransfer {
    type ActorState = ActorState;
    type Error = Error;
    type State = State;

    async fn process(&self, state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        let generated = state.generate_login_token(self.account_id, self.realm_id)?;
        let mut msg = self.clone();
        msg.token = generated.token;
        actor.send(msg).await?;
        actor.shutdown().await?;
        Ok(())
    }
}
