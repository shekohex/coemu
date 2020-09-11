use super::{MsgTalk, TalkChannel};
use crate::{utils, ActorState, Error};
use async_trait::async_trait;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};
use tq_network::{Actor, IntoErrorPacket, PacketID, PacketProcess};
use tracing::{debug, warn};
use utils::LoHi;
#[derive(Debug, FromPrimitive)]
#[repr(u16)]
pub enum ActionType {
    #[num_enum(default)]
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
    Revive = 94,
    DelRole = 95,
    SetKillMode = 96,
    ConfirmGuild = 97,
    Mine = 99,
    /// [101]
    /// Data2 = TeamMemberId,
    /// Data3Low = PositionX,
    /// Data3High = PositionY
    TeamMemberPos = 101,
    QueryEntity = 102,
    AbortMagic = 103,
    MapARGB = 104,
    MapStatus = 105,
    /// [106]
    /// Data3Low = PositionX,
    /// Data3High = PositionY
    QueryTeamMember = 106,
    Kickback = 108,
    DropMagic = 109,
    DropSkill = 110,
    /// [111]
    /// Data2 = BoothId,
    /// Data3Low = PositionX,
    /// Data3High = PositionY,
    /// Data4 = Direction
    CreateBooth = 111,
    SuspendBooth = 112,
    ResumeBooth = 113,
    LeaveBooth = 114,
    PostCommand = 116,
    /// [117]
    /// Data2 = TargetId
    QueryEquipment = 117,
    AbortTransform = 118,
    EndFly = 120,
    /// [121]
    /// Data2
    GetMoney = 121,
    QueryEnemy = 123,
    OpenDialog = 126,
    LogainCompeleted = 130,
    LeaveMap = 132,
    GroundJump = 133,
    /// [134]
    /// Data1 = EntityId,
    /// Data3Low = PositionX,
    /// Data3High = PositionY
    SpawnEffect = 134,
    /// [135]
    /// Data1 = EntityId
    RemoveEntity = 135,
    Jump = 137,
    TeleportReply = 138,
    DeathConfirmation = 145,
    /// [148]
    /// Data1 = FriendId
    QueryAssociateInfo = 148,
    ChangeFace = 151,
    ItemsDetained = 155,
    NinjaStep = 156,
    HideInterface = 158,
    OpenUpgrade = 160,
    /// [161]
    /// Data1 = Mode (0=none,1=away)
    AwayFromKeyboard = 161,
    PathFinding = 162,
    DragonBallDropped = 165,
    TableState = 233,
    TablePot = 234,
    TablePlayerCount = 235,
    /// [310]
    /// Data2 = FriendId
    QueryFriendEquip = 310,
    QueryStatInfo = 408,
}

/// Message containing a general action being performed by the client. Commonly
/// used as a request-response protocol for question and answer like exchanges.
/// For example, walk requests are responded to with an answer as to if the step
/// is legal or not.
#[derive(Debug, Default, Serialize, Deserialize, Clone, PacketID)]
#[packet(id = 1010)]
pub struct MsgAction {
    client_timestamp: u32,
    character_id: u32,
    data1: u32,
    data2: u32,
    details: u16,
    action_type: u16,
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

    async fn handle_send_location(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let mut res = self.clone();
        let character = actor.character().await?;
        res.data1 = character.map_id();
        res.data2 = u32::constract(character.y(), character.x());
        actor.send(res).await?;
        // TODO(shekohex): send MsgMapInfo
        Ok(())
    }

    async fn handle_map_argb(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let mut res = self.clone();
        let character = actor.character().await?;
        res.data1 = 0x00FF_FFFF;
        res.data2 = u32::constract(character.y(), character.x());
        actor.send(res).await?;
        Ok(())
    }

    async fn handle_leave_booth(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        // Remove Player from Booth.
        let myscreen = actor.screen().await?;
        myscreen.clear().await?;
        myscreen.load_surroundings().await?;
        Ok(())
    }

    async fn handle_jump(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let new_x = self.data1.lo();
        let new_y = self.data1.hi();
        let current_x = self.data2.lo();
        let current_y = self.data2.hi();
        let me = actor.character().await?;
        // Starting to validate this jump.
        if current_x != me.x() || current_y != me.y() {
            debug!(
                "Bad Packet Got ({}, {}) but expected ({}, {})",
                current_x,
                current_y,
                me.x(),
                me.y()
            );
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

        let mymap = actor.map().await?;
        let within_elevation = mymap
            .sample_elevation((me.x(), me.y()), (new_x, new_y), me.elevation())
            .await;
        if !within_elevation {
            debug!(
                "Cannot jump that high. new elevation {} diff > 210",
                me.elevation()
            );
            me.kick_back().await?;
            return Ok(());
        }

        // I guess everything seems to be valid .. send the jump.

        let direction =
            tq_math::get_direction_sector((me.x(), me.y()), (new_x, new_y));
        let tile = mymap.tile(new_x, new_y).await.ok_or_else(|| {
            MsgTalk::from_system(
                me.id(),
                TalkChannel::TopLeft,
                String::from("Invalid Location"),
            )
            .error_packet()
        })?;
        me.set_x(new_x)
            .set_y(new_y)
            .set_direction(direction)
            .set_action(100);
        me.set_elevation(tile.elevation);
        actor.send(self.clone()).await?;
        let myscreen = actor.screen().await?;
        myscreen.send_movement(self.clone()).await?;
        Ok(())
    }

    async fn handle_change_facing(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let current_x = self.data2.lo();
        let current_y = self.data2.hi();
        let me = actor.character().await?;

        // Starting to validate this jump.
        if current_x != me.x() || current_y != me.y() {
            // Kick Back.
            me.kick_back().await?;
            return Ok(());
        }

        me.set_direction(self.details as u8);
        actor.send(self.clone()).await?;
        let myscreen = actor.screen().await?;
        myscreen.send_message(self.clone()).await?;
        Ok(())
    }

    async fn handle_query_entity(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let mymap = actor.map().await?;
        let characters = mymap.characters().read().await;
        let other = characters.get(&self.data1);
        if let Some(other) = other {
            let msg = super::MsgPlayer::from(other.clone());
            actor.send(msg).await?;
        }
        Ok(())
    }

    async fn handle_change_map(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        let portal_x = self.data1.lo();
        let portal_y = self.data1.hi();
        let me = actor.character().await?;
        if !tq_math::in_screen((me.x(), me.y()), (portal_x, portal_y)) {
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
        dbg!(portal_x, portal_y);
        let mymap = actor.map().await?;
        let maybe_portal = mymap.portals().iter().find(|p| {
            tq_math::in_circle((me.x(), me.y(), 15), (p.from_x(), p.from_y()))
        });
        if let Some(portal) = maybe_portal {
            dbg!(portal);
            me.teleport(portal.to_map_id(), (portal.to_x(), portal.to_y()))
                .await?;
        } else {
            // TODO
            me.teleport(me.map_id(), (me.prev_x(), me.prev_y())).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl PacketProcess for MsgAction {
    type ActorState = ActorState;
    type Error = Error;

    async fn process(
        &self,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let ty = self.action_type.into();
        match ty {
            ActionType::SendLocation => self.handle_send_location(actor).await,
            ActionType::MapARGB => self.handle_map_argb(actor).await,
            ActionType::LeaveBooth => self.handle_leave_booth(actor).await,
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
            ActionType::LogainCompeleted => Ok(()),
            ActionType::GroundJump => self.handle_jump(actor).await,
            ActionType::ChangeFacing => self.handle_change_facing(actor).await,
            ActionType::QueryEntity => self.handle_query_entity(actor).await,
            ActionType::ChangeMap => self.handle_change_map(actor).await,
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
