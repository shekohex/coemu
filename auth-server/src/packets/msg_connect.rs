use crate::Error;
use async_trait::async_trait;
use network::{Actor, PacketID, PacketProcess};
use serde::Deserialize;
use tq_serde::String16;
/// Message containing a connection request to the game server. Contains the
/// player's access token from the Account server, and the patch and language
/// versions of the game client.
#[derive(Debug, Deserialize, PacketID)]
#[packet(id = 1052)]
pub struct MsgConnect {
    id: u32,
    file_contents: u32,
    file_name: String16,
}

#[async_trait]
impl PacketProcess for MsgConnect {
    type Error = Error;

    async fn process(&self, actor: &Actor) -> Result<(), Self::Error> {
        actor.shutdown().await?;
        Ok(())
    }
}
