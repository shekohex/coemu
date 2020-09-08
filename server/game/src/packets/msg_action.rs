use super::{MsgTalk, TalkChannel};
use crate::{systems::TileType, utils, ActorState};
use async_trait::async_trait;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};
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
}

#[async_trait]
impl PacketProcess for MsgAction {
    type ActorState = ActorState;
    type Error = crate::Error;

    async fn process(
        &self,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let ty = self.action_type.into();
        match ty {
            ActionType::SendLocation => {
                let mut res = self.clone();
                let character = actor.character().await?;
                res.data1 = character.map_id();
                res.data2 = u32::constract(character.y(), character.x());
                actor.send(res).await?;
                // TODO(shekohex): send MsgMapInfo
            },
            ActionType::MapARGB => {
                let mut res = self.clone();
                let character = actor.character().await?;
                res.data1 = 0x00FF_FFFF;
                res.data2 = u32::constract(character.y(), character.x());
                actor.send(res).await?;
            },
            ActionType::LeaveBooth => {
                // Remove Player from Booth.
                let myscreen = actor.screen().await?;
                myscreen.clear().await?;
                myscreen.load_surroundings().await?;
            },
            ActionType::SendItems => {
                // TODO(shekohex): send MsgItemInfo
            },
            ActionType::SendAssociates => {
                // Friends.
                // TODO(shekohex): send MsgFriend
            },
            ActionType::SendProficiencies => {
                // Skils
                // TODO(shekohex): send MsgWeaponSkill
            },
            ActionType::SendSpells => {
                // Magic Spells
                // TODO(shekohex): send MsgMagicInfo
            },
            ActionType::ConfirmGuild => {
                // TODO(shekohex): send MsgSyndicateAttributeInfo
            },
            ActionType::LogainCompeleted => {
                // Login Completed
            },
            ActionType::GroundJump => {
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
                let tile = mymap.tile(new_x, new_y).await?;
                if tile.access < TileType::Npc {
                    debug!("Cannot jump on that tile = {:?}. Type ({:?} = {}) < Npc", tile, tile.access, tile.access as u8);
                    me.kick_back().await?;
                    return Ok(());
                }

                if !tq_math::within_elevation(tile.elevation, me.elevation()) {
                    debug!("Cannot jump that high. new elevation {} but current {} diff > 210", tile.elevation, me.elevation());
                    me.kick_back().await?;
                    return Ok(());
                }

                // I guess everything seems to be valid .. send the jump.

                let direction = tq_math::get_direction_sector(
                    (me.x(), me.y()),
                    (new_x, new_y),
                );
                me.set_x(new_x)
                    .set_y(new_y)
                    .set_direction(direction)
                    .set_action(100);
                actor.send(self.clone()).await?;
                let myscreen = actor.screen().await?;
                myscreen.send_movement(self.clone()).await?;
            },
            ActionType::ChangeFacing => {
                let current_x = self.data2.lo();
                let current_y = self.data2.hi();
                let me = actor.character().await?;

                // Starting to validate this jump.
                if current_x != me.x() || current_y != me.y() {
                    // Kick Back.
                }

                me.set_direction(self.details as u8);
                actor.send(self.clone()).await?;
                let myscreen = actor.screen().await?;
                myscreen.send_message(self.clone()).await?;
            },
            ActionType::QueryEntity => {
                let mymap = actor.map().await?;
                let other = mymap.characters().get(&self.data1);
                if let Some(other) = other {
                    let msg = super::MsgPlayer::from(other.clone());
                    actor.send(msg).await?;
                }
            },
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
            },
        };
        Ok(())
    }
}
