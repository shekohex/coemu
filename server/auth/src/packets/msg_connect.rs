use crate::state::State;
use crate::Error;
use serde::Deserialize;
use tq_network::{Actor, PacketID, PacketProcess};
use tq_serde::String16;

/// Message containing a connection request to the game server. Contains the
/// player's access token from the Account server, and the patch and language
/// versions of the game client.
#[derive(Debug, Deserialize, PacketID)]
#[packet(id = 1052)]
#[allow(dead_code)]
pub struct MsgConnect {
    id: u32,
    file_contents: u32,
    file_name: String16,
}

#[async_trait::async_trait]
impl PacketProcess for MsgConnect {
    type ActorState = ();
    type Error = Error;
    type State = State;

    async fn process(
        &self,
        _state: &Self::State,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        actor.shutdown().await?;
        Ok(())
    }
}
