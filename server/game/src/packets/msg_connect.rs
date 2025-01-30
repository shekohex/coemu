use super::{MsgTalk, MsgUserInfo};
use crate::entities::Character;
use crate::packets::MsgData;
use crate::systems::Screen;
use crate::{ActorState, Error, State};
use serde::{Deserialize, Serialize};
use tq_network::{Actor, IntoErrorPacket, PacketID, PacketProcess};
use tq_serde::String10;

/// Message containing a connection request to the game server. Contains the
/// player's access token from the Account server, and the patch and language
/// versions of the game client.
#[derive(Debug, Default, Serialize, Deserialize, PacketID)]
#[packet(id = 1052)]
#[allow(dead_code)]
pub struct MsgConnect {
    pub token: u64,
    pub build_version: u16,
    pub language: String10,
    pub file_contents: u32,
}

#[async_trait::async_trait]
impl PacketProcess for MsgConnect {
    type ActorState = ActorState;
    type Error = Error;
    type State = State;

    async fn process(&self, state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        let info = state
            .remove_login_token(self.token)
            .map_err(|_| MsgTalk::login_invalid().error_packet())?;
        actor.generate_keys(self.token).await?;
        actor.set_id(info.account_id as usize);
        let maybe_character = tq_db::character::Character::from_account(state.pool(), info.account_id).await?;
        match maybe_character {
            Some(character) => {
                let me = Character::new(actor.handle(), character);
                let mymap_id = me.entity().map_id();
                let screen = Screen::new(actor.handle());
                let msg = MsgUserInfo::from(&me);
                actor.update(me, screen);
                let mymap = state
                    .try_map(mymap_id)
                    .map_err(|_| MsgTalk::login_invalid().error_packet())?;
                mymap.insert_entity(actor.entity()).await?;
                state.insert_entity(actor.entity());
                actor.send(MsgTalk::login_ok()).await?;
                actor.send(msg).await?;
                actor.send(MsgData::now()).await?;
            },
            None => {
                state.store_creation_token(self.token as u32, info.account_id, info.realm_id)?;
                actor.send(MsgTalk::login_new_role()).await?;
            },
        };
        Ok(())
    }
}
