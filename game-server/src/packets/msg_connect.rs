use super::{MsgTalk, MsgUserInfo, TalkChannel};
use crate::constants::ANSWER_OK;
use async_trait::async_trait;
use network::{Actor, PacketProcess};
use serde::Deserialize;
use tq_serde::String10;

/// Message containing a connection request to the game server. Contains the
/// player's access token from the Account server, and the patch and language
/// versions of the game client.
#[derive(Debug, Default, Deserialize)]
pub struct MsgConnect {
    authentication_token: u32,
    authentication_code: u32,
    build_version: u16,
    language: String10,
    file_contents: u32,
}

#[async_trait]
impl PacketProcess for MsgConnect {
    type Error = crate::Error;

    async fn process(&self, actor: &Actor) -> Result<(), Self::Error> {
        actor
            .generate_keys(self.authentication_code, self.authentication_token)
            .await?;
        let msg =
            MsgTalk::from_system(0, TalkChannel::Login, ANSWER_OK.to_string());
        actor.send(msg).await?;
        let msg = MsgUserInfo::default();
        actor.send(msg).await?;
        Ok(())
    }
}
