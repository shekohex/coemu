use super::{MsgTalk, MsgUserInfo};
use crate::{db, systems::Screen, world::Character, ActorState, Error, State};
use async_trait::async_trait;
use serde::Deserialize;
use tq_network::{Actor, IntoErrorPacket, PacketID, PacketProcess};
use tq_serde::String10;

/// Message containing a connection request to the game server. Contains the
/// player's access token from the Account server, and the patch and language
/// versions of the game client.
#[derive(Debug, Default, Deserialize, PacketID)]
#[packet(id = 1052)]
pub struct MsgConnect {
    token: u32,
    code: u32,
    build_version: u16,
    language: String10,
    file_contents: u32,
}

#[async_trait]
impl PacketProcess for MsgConnect {
    type ActorState = ActorState;
    type Error = Error;

    async fn process(
        &self,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let state = State::global()?;
        let (id, realm_id) = state
            .login_tokens()
            .remove(&self.token)
            .map(|(_, account_id)| account_id)
            .ok_or_else(|| MsgTalk::login_invalid().error_packet())?;
        actor.generate_keys(self.code, self.token).await?;
        actor.set_id(id as usize);
        let maybe_character = db::Character::from_account(id).await?;
        match maybe_character {
            Some(character) => {
                let me = Character::new(actor.clone(), character.clone());
                actor.set_character(me.clone()).await?;
                state
                    .maps()
                    .get(&(character.map_id as u32))
                    .ok_or_else(|| MsgTalk::login_invalid().error_packet())?
                    .insert_character(me)
                    .await?;
                let screen = Screen::new(actor.clone());
                actor.set_screen(screen).await?;
                actor.send(MsgTalk::login_ok()).await?;
                let msg = MsgUserInfo::from(character);
                actor.send(msg).await?;
            },
            None => {
                state.creation_tokens().insert(self.token, (id, realm_id));
                actor.send(MsgTalk::login_new_role()).await?;
            },
        };
        Ok(())
    }
}
