use super::{MsgTalk, TalkChannel};
use crate::entities::Character;
use crate::packets::{MsgMapInfo, MsgWeather};
use crate::state::State;
use crate::systems::TileType;
use crate::{utils, ActorState, Error};
use async_trait::async_trait;
use num_enum::{FromPrimitive, IntoPrimitive};
use primitives::Location;
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};
use utils::LoHi;

#[derive(Copy, Clone, Debug, Default, FromPrimitive, IntoPrimitive)]
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

#[derive(Copy, Clone, Debug, Default, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum KillMode {
    #[default]
    Free = 0,
    Safe = 1,
    Team = 2,
    Arrestment = 3,
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
    pub fn new(character_id: u32, data1: u32, data2: u32, details: u16, action_type: ActionType) -> Self {
        Self {
            client_timestamp: utils::current_ts(),
            character_id,
            data1,
            data2,
            details,
            action_type: action_type as u16,
        }
    }

    pub fn from_character(character: &Character, data1: u32, action_type: ActionType) -> Self {
        let character_id = character.id();
        let (x, y, d) = character.entity().location().into();
        let data2 = u32::constract(y, x);
        let details = d as u16;
        Self::new(character_id, data1, data2, details, action_type)
    }

    #[tracing::instrument(skip_all)]
    async fn handle_send_location(&self, state: &State, actor: &Actor<ActorState>) -> Result<(), Error> {
        let mut res = self.clone();
        let entity = actor.try_entity()?;
        let character = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let map_id = character.entity().map_id();
        let location = character.entity().location();
        match state.try_map(map_id) {
            Ok(mymap) => {
                res.data1 = map_id;
                res.data2 = u32::constract(location.y, location.x);
                mymap.insert_entity(entity).await?;
                actor.send(res).await?;
                actor.send(MsgMapInfo::from_map(mymap)).await?;
                if !mymap.weather().is_unknwon() {
                    actor.send(MsgWeather::new(mymap.weather())).await?;
                }
                let screen = actor.screen();
                screen.load_surroundings(state).await?;
            },
            Err(_) => {
                tracing::warn!(
                    character_id = character.id(),
                    %map_id,
                    "map not found",
                );
                // Set default map and location.
                let location = Location::new(430, 378, 0);
                character.entity().set_map_id(1002).set_location(location);
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
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_map_argb(&self, state: &State, actor: &Actor<ActorState>) -> Result<(), Error> {
        let mut res = self.clone();
        let entity = actor.try_entity()?;
        let character = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let map_id = character.entity().map_id();
        let location = character.entity().location();
        let map = state.try_map(map_id)?;
        res.data1 = map.color();
        res.data2 = u32::constract(location.y, location.x);
        actor.send(res).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_login_completed(&self, _state: &State, actor: &Actor<ActorState>) -> Result<(), Error> {
        let res = self.clone();
        actor.send(res).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_leave_booth(&self, state: &State, actor: &Actor<ActorState>) -> Result<(), Error> {
        // Remove Player from Booth.
        let myscreen = actor.screen();
        myscreen.clear()?;
        myscreen.load_surroundings(state).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_jump(&self, state: &State, actor: &Actor<ActorState>) -> Result<(), Error> {
        let new_x = self.data1.lo();
        let new_y = self.data1.hi();
        let current_x = self.data2.lo();
        let current_y = self.data2.hi();
        let entity = actor.try_entity()?;
        let me = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let loc = me.entity().location();
        let mymap_id = me.entity().map_id();
        // Starting to validate this jump.
        if current_x != loc.x || current_y != loc.y {
            tracing::debug!(%current_x, %current_y, %loc.x, %loc.y, "Bad Jump Packet");
            me.kick_back().await?;
            return Ok(());
        }

        if !tq_math::in_screen((loc.x, loc.y), (new_x, new_y)) {
            tracing::debug!(%loc.x, %loc.y, %new_x, %new_y, "Bad Location Distance > 18");
            me.kick_back().await?;
            return Ok(());
        }

        let mymap = state.try_map(mymap_id)?;
        let within_elevation = mymap.sample_elevation((loc.x, loc.y), (new_x, new_y), me.elevation());
        if !within_elevation {
            tracing::debug!(%loc.x, %loc.y, %new_x, %new_y, "Elevation diff > 210");
            me.kick_back().await?;
            return Ok(());
        }

        let direction = tq_math::get_direction_sector((loc.x, loc.y), (new_x, new_y));
        match mymap.tile(new_x, new_y) {
            Some(tile) if tile.access > TileType::Npc => {
                // I guess everything seems to be valid .. send the jump.
                me.entity()
                    .set_location(Location::new(new_x, new_y, direction))
                    .set_action(100);
                me.set_elevation(tile.elevation);
                mymap.update_region_for(entity.clone());
                actor.send(self.clone()).await?;
                let myscreen = actor.screen();
                myscreen.send_movement(state, self.clone()).await?;
            },
            Some(_) | None => {
                // Invalid Location move them back
                let msg = MsgTalk::from_system(me.id(), TalkChannel::TopLeft, String::from("Invalid Location"));
                actor.send(msg).await?;
                me.kick_back().await?;
                tracing::debug!(id = %me.id(), %loc.x, %loc.y, %new_x, %new_y, "Invalid Location");
                return Ok(());
            },
        };
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_change_facing(&self, actor: &Actor<ActorState>) -> Result<(), Error> {
        let current_x = self.data2.lo();
        let current_y = self.data2.hi();
        let entity = actor.try_entity()?;
        let me = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let mut loc = me.entity().location();

        // Starting to validate this action.
        if current_x != loc.x || current_y != loc.y {
            // Kick Back.
            me.kick_back().await?;
            return Ok(());
        }

        loc.direction = self.details as u8;
        me.entity().set_location(loc);
        actor.send(self.clone()).await?;
        let myscreen = actor.screen();
        myscreen.send_message(self.clone()).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_query_entity(&self, state: &State, actor: &Actor<ActorState>) -> Result<(), Error> {
        let entity = actor.try_entity()?;
        let me = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let mymap_id = me.entity().map_id();
        let mymap = state.try_map(mymap_id)?;
        let other = mymap.with_regions(|r| r.iter().find_map(|r| r.try_entities(self.data1)));
        if let Some(other) = other.and_then(|o| o.upgrade()).as_ref().and_then(|o| o.as_character()) {
            let msg = super::MsgPlayer::from(other);
            actor.send(msg).await?;
        } else {
            let msg = super::MsgTalk::from_system(me.id(), TalkChannel::System, "Player not found");
            actor.send(msg).await?;
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_change_map(&self, state: &State, actor: &Actor<ActorState>) -> Result<(), Error> {
        let portal_x = self.data1.lo();
        let portal_y = self.data1.hi();
        let entity = actor.try_entity()?;
        let me = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let loc = me.entity().location();
        let mymap_id = me.entity().map_id();
        if !tq_math::in_screen((loc.x, loc.y), (portal_x, portal_y)) {
            // TODO: Jail for using Portal Hack.
            tracing::debug!(%portal_x, %portal_y, %loc.x, %loc.y, "Using Portal Hack");
            me.kick_back().await?;
            return Ok(());
        }
        let mymap = state.try_map(mymap_id)?;
        let maybe_portal = mymap
            .portals()
            .iter()
            .find(|p| tq_math::in_circle((loc.x, loc.y, 5), (p.from_x(), p.from_y())));
        match maybe_portal {
            Some(portal) => {
                let portal_map = state.try_map(portal.to_map_id())?;
                mymap.remove_entity(&entity)?;
                portal_map.insert_entity(entity.clone()).await?;
                me.teleport(state, portal.to_map_id(), (portal.to_x(), portal.to_y()))
                    .await?;
            },
            None => {
                tracing::debug!(%portal_x, %portal_y, %loc.x, %loc.y, "Portal not found");
                me.kick_back().await?;
            },
        }
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn handle_set_kill_mode(&self, _state: &State, actor: &Actor<ActorState>) -> Result<(), Error> {
        let kill_mode = KillMode::from(self.data1 as u16);
        // TODO: Update player kill mode.
        // TODO: handle i18n
        let notice = match kill_mode {
            KillMode::Free => "In free mode, you can attack everybody.",
            KillMode::Safe => "In safe mode, you can only attack monsters.",
            KillMode::Team => {
                "In team mode, you can attack everybody, except your friends, your teammates, and your guildmates."
            },
            KillMode::Arrestment => "In arrestment mode, you can only attack monsters and black name players.",
        };
        actor.send(self.clone()).await?;
        let msg = super::MsgTalk::from_system(actor.entity().id(), TalkChannel::System, notice);
        actor.send(msg).await?;
        Ok(())
    }
}

#[async_trait]
impl PacketProcess for MsgAction {
    type ActorState = ActorState;
    type Error = Error;
    type State = State;

    async fn process(&self, state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        let ty = self.action_type.into();
        match ty {
            ActionType::SendLocation => self.handle_send_location(state, actor).await,
            ActionType::MapARGB => self.handle_map_argb(state, actor).await,
            ActionType::SetKillMode => self.handle_set_kill_mode(state, actor).await,
            ActionType::LeaveBooth => self.handle_leave_booth(state, actor).await,
            ActionType::SendItems => {
                // TODO: send MsgItemInfo
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::SendAssociates => {
                // Friends.
                // TODO: send MsgFriend
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::SendProficiencies => {
                // Skils
                // TODO: send MsgWeaponSkill
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::SendSpells => {
                // Magic Spells
                // TODO: send MsgMagicInfo
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::ConfirmGuild => {
                // TODO: send MsgSyndicateAttributeInfo
                actor.send(self.clone()).await?;
                Ok(())
            },
            ActionType::LoginCompeleted => self.handle_login_completed(state, actor).await,
            ActionType::Jump => self.handle_jump(state, actor).await,
            ActionType::ChangeFacing => self.handle_change_facing(actor).await,
            ActionType::QueryEntity => self.handle_query_entity(state, actor).await,
            ActionType::ChangeMap => self.handle_change_map(state, actor).await,
            _ => {
                let p = MsgTalk::from_system(
                    self.character_id,
                    TalkChannel::Talk,
                    format!("Missing Action Type {ty:?} = {}", self.action_type),
                );
                actor.send(p).await?;
                let res = self.clone();
                actor.send(res).await?;
                tracing::warn!("Missing Action Type {ty:?} = {}", self.action_type);
                Ok(())
            },
        }
    }
}
