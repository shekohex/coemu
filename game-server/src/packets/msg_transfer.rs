use crate::Error;
use async_trait::async_trait;
use network::{Actor, PacketID, PacketProcess};
use serde::{Deserialize, Serialize};

/// Defines account parameters to be transferred from the account server to the
/// game server. Account information is supplied from the account database, and
/// used on the game server to transfer authentication and authority level.  
#[derive(Clone, Debug, Deserialize, Serialize, PacketID)]
#[packet(id = 1000)]
pub struct MsgTransfer {
    account_id: u32,
    #[serde(skip_deserializing)]
    authentication_token: u32,
    #[serde(skip_deserializing)]
    authentication_code: u32,
}

#[async_trait]
impl PacketProcess for MsgTransfer {
    type Error = Error;

    async fn process(&self, actor: &Actor) -> Result<(), Self::Error> {
        let mut msg = self.clone();
        msg.authentication_token = 1001;
        msg.authentication_code = 1002;
        actor.send(msg).await?;
        actor.shutdown().await?;
        Ok(())
    }
}
