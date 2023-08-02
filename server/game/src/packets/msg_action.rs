use super::{MsgTalk, TalkChannel};
use crate::packets::{MsgMapInfo, MsgWeather};
use crate::state::State;
use crate::systems::TileType;
use crate::{utils, ActorState, Error};
use async_trait::async_trait;
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};
use tracing::{debug, warn};
use utils::LoHi;
#[derive(Debug, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ActionType {
    #[default]
    Unknown = 0,
    SendLocation = 74,
    SendItems = 75,
    SendAssociates = 76,
    SendProficiencies = 77,
    SendSpells = 78,
    ChangeFacing = 79,
    ChangeAction = 81,
    ChangeMap = 85,
    Teleport = 86,
    LevelUp = 92,
    XpClear = 93,
    Reborn = 94,
    DelRole = 95,
    SetKillMode = 96,
    ConfirmGuild = 97,
    Mine = 99,
    /// Not sure of the name...
    BotCheckA = 100,
    QueryEntity = 102,
    /// to client only
    MapARGB = 104,
    QueryTeamMember = 106,
    /// to client idUser is Player ID, unPosX unPosY is Player pos
    KickBack = 108,
    // to client only, data is magic type
    DropMagic = 109,
    // to client only, data is weapon skill type
    DropSkill = 110,
    CreateBooth = 111,
    SuspendBooth = 112,
    ResumeBooth = 113,
    LeaveBooth = 114,
    PostCommand = 116,
    QueryEquipment = 117,
    AbortTransform = 118,
    // CombineSubSyn = 119,
    // idTargetSyn
    TakeOff = 120,
    GetMoney = 121,
    CancelKeepBow = 122,
    QueryEnemyInfo = 123,
    OpenDialog = 126,
    // FlashStatus = 127,
    LoginCompeleted = 130,
    LeaveMap = 132,
    Jump = 133,
    Ghost = 137,
    Synchro = 138,
    QueryFriendInfo = 140,
    // QueryLeaveWord = 141,
    ChangeFace = 142,
}

/// Message containing a general action being performed by the client. Commonly
/// used as a request-response protocol for question and answer like exchanges.
/// For example, walk requests are responded to with an answer as to if the step
/// is legal or not.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PacketID)]
#[packet(id = 1010)]
pub struct MsgAction {
    pub client_timestamp: u32,
    pub character_id: u32,
    pub data1: u32,
    pub data2: u32,
    pub details: u16,
    pub action_type: u16,
}

impl MsgAction {
    pub fn new(
        character_id: u32,
        data1: u32,
        data2: u32,
        details: u16,
        action_type: ActionType,
    ) -> Self {
        Self {
            client_timestamp: utils::current_ts(),
            character_id,
            data1,
            data2,
            details,
            action_type: action_type as u16,
        }
    }

    #[tracing::instrument(skip_all)]
    async fn handle_send_location(
        &self,
        state: &State,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let mut res = self.clone();
        let character = actor.character();
        match state.try_map(character.map_id()) {
            Ok(mymap) => {
                res.data1 = character.map_id();
                res.data2 = u32::constract(character.y(), character.x());
                mymap.insert_character(character).await?;
                actor.send(res).await?;
                actor.send(MsgMapInfo::from_map(mymap)).await?;
                if mymap.weather != 0 {
                    actor
                        .send(MsgWeather::new((mymap.weather as u32).into()))
                        .await?;
                }
                let screen = actor.screen();
                screen.load_surroundings(state).await?;
            },
            Err(_) => {
                warn!(
                    character_id = character.id(),
                    map_id = character.map_id(),
                    "map not found",
                );
                // Set default map and location.
                character.set_map_id(1002).set_x(430).set_y(378);
                actor
                    .send(MsgTalk::from_system(
                        character.id(),
                        TalkChannel::System,
                        "Map not found, teleporting to Twin City.".to_string(),
                    ))
                    .await?;
                actor.shutdown().await?;
                return Ok(());
            },
        };
        // TODO(shekohex): send MsgMapInfo
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_map_argb(
        &self,
        state: &State,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let mut res = self.clone();
        let character = actor.character();
        let map_id = character.map_id();
        let map = state.try_map(map_id)?;
        res.data1 = map.color as _;
        res.data2 = u32::constract(character.y(), character.x());
        actor.send(res).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_login_completed(
        &self,
        _state: &State,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let res = self.clone();
        actor.send(res).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_leave_booth(
        &self,
        state: &State,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        // Remove Player from Booth.
        let myscreen = actor.screen();
        myscreen.clear()?;
        myscreen.load_surroundings(state).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_jump(
        &self,
        state: &State,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let new_x = self.data1.lo();
        let new_y = self.data1.hi();
        let current_x = self.data2.lo();
        let current_y = self.data2.hi();
        let me = actor.character();
        // Starting to validate this jump.
        if current_x != me.x() || current_y != me.y() {
            debug!(%current_x, %current_y, my_x = %me.x(), my_y = %me.y(),"Bad Jump Packet");
            me.kick_back().await?;
            return Ok(());
        }

        if !tq_math::in_screen((me.x(), me.y()), (new_x, new_y)) {
            debug!(
                "Bad Location ({}, {}) -> ({}, {}) > 18",
                new_x,
                new_y,
                me.x(),
                me.y()
            );
            me.kick_back().await?;
            return Ok(());
        }

        let mymap = state.try_map(me.map_id())?;
        let within_elevation = mymap.sample_elevation(
            (me.x(), me.y()),
            (new_x, new_y),
            me.elevation(),
        );
        if !within_elevation {
            debug!(
                "Cannot jump that high. new elevation {} diff > 210",
                me.elevation()
            );
            me.kick_back().await?;
            return Ok(());
        }

        let direction =
            tq_math::get_direction_sector((me.x(), me.y()), (new_x, new_y));
        match mymap.tile(new_x, new_y) {
            Some(tile) if tile.access > TileType::Npc => {
                // I guess everything seems to be valid .. send the jump.
                me.set_x(new_x)
                    .set_y(new_y)
                    .set_direction(direction)
                    .set_action(100);
                me.set_elevation(tile.elevation);
                mymap.update_region_for(me.clone());
                actor.send(self.clone()).await?;
                let myscreen = actor.screen();
                myscreen.send_movement(state, self.clone()).await?;
            },
            Some(_) | None => {
                // Invalid Location move them back
                let msg = MsgTalk::from_system(
                    me.id(),
                    TalkChannel::TopLeft,
                    String::from("Invalid Location"),
                );
                actor.send(msg).await?;
                me.kick_back().await?;
                tracing::debug!(id = %me.id(), x = %me.x(), y = %me.y(), %new_x, %new_y, "Invalid Location");
                return Ok(());
            },
        };
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_change_facing(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let current_x = self.data2.lo();
        let current_y = self.data2.hi();
        let me = actor.character();

        // Starting to validate this action.
        if current_x != me.x() || current_y != me.y() {
            // Kick Back.
            me.kick_back().await?;
            return Ok(());
        }

        me.set_direction(self.details as u8);
        actor.send(self.clone()).await?;
        let myscreen = actor.screen();
        myscreen.send_message(self.clone()).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_query_entity(
        &self,
        state: &State,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let me = actor.character();
        let mymap = state.try_map(me.map_id())?;
        let other = mymap.with_regions(|r| {
            r.iter().find_map(|r| r.try_character(self.data1))
        });
        if let Some(other) = other.and_then(|o| o.upgrade()) {
            let msg = super::MsgPlayer::from(other.as_ref());
            actor.send(msg).await?;
        } else {
            let msg = super::MsgTalk::from_system(
                me.id(),
                TalkChannel::System,
                "Player not found",
            );
            actor.send(msg).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_change_map(
        &self,
        state: &State,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let portal_x = self.data1.lo();
        let portal_y = self.data1.hi();
        let me = actor.character();
        if !tq_math::in_screen((me.x(), me.y()), (portal_x, portal_y)) {
            // TODO: Jail for using Portal Hack.
            debug!(
                "Bad Location ({}, {}) -> ({}, {}) > 18",
                portal_x,
                portal_y,
                me.x(),
                me.y()
            );
            me.kick_back().await?;
            return Ok(());
        }
        let mymap = state.try_map(me.map_id())?;
        let maybe_portal = mymap.portals().iter().find(|p| {
            tq_math::in_circle((me.x(), me.y(), 5), (p.from_x(), p.from_y()))
        });
        if let Some(portal) = maybe_portal {
            let portal_map = state.try_map(portal.to_map_id())?;
            portal_map.insert_character(me.clone()).await?;
            me.teleport(
                state,
                portal.to_map_id(),
                (portal.to_x(), portal.to_y()),
            )
            .await?;
            mymap.remove_character(&me)?;
        } else {
            me.teleport(state, me.map_id(), (me.prev_x(), me.prev_y()))
                .await?;
        }
        Ok(())
    }
}

#[async_trait]
impl PacketProcess for MsgAction {
    type ActorState = ActorState;
    type Error = Error;
    type State = State;

    async fn process(
        &self,
        state: &Self::State,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let ty = self.action_type.into();
        match ty {
            ActionType::SendLocation => {
                self.handle_send_location(state, actor).await
            },
            ActionType::MapARGB => self.handle_map_argb(state, actor).await,
            ActionType::LeaveBooth => {
                self.handle_leave_booth(state, actor).await
            },
            ActionType::SendItems => {
                // TODO(shekohex): send MsgItemInfo
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::SendAssociates => {
                // Friends.
                // TODO(shekohex): send MsgFriend
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::SendProficiencies => {
                // Skils
                // TODO(shekohex): send MsgWeaponSkill
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::SendSpells => {
                // Magic Spells
                // TODO(shekohex): send MsgMagicInfo
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::ConfirmGuild => {
                // TODO(shekohex): send MsgSyndicateAttributeInfo
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::LoginCompeleted => {
                self.handle_login_completed(state, actor).await
            },
            ActionType::Jump => self.handle_jump(state, actor).await,
            ActionType::ChangeFacing => self.handle_change_facing(actor).await,
            ActionType::QueryEntity => {
                self.handle_query_entity(state, actor).await
            },
            ActionType::ChangeMap => self.handle_change_map(state, actor).await,
            _ => {
                let p = MsgTalk::from_system(
                    self.character_id,
                    TalkChannel::Talk,
                    format!(
                        "Missing Action Type {:?} = {}",
                        ty, self.action_type
                    ),
                );
                actor.send(p).await?;
                let res = self.clone();
                actor.send(res).await?;
                warn!("Missing Action Type {:?} = {}", ty, self.action_type);
                Ok(())
            },
        }
    }
}
