use crate::{
    constants::{ALL_USERS, SYSTEM},
    ActorState,
};
use async_trait::async_trait;
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use tq_network::{Actor, IntoErrorPacket, PacketID, PacketProcess};
use tracing::warn;

/// Enumeration for defining the channel text is printed to. Can also print to
/// separate states of the client such as character registration, and can be
/// used to change the state of the client or deny a login.
#[derive(Copy, Clone, Debug, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum TalkChannel {
    Talk = 2000,
    Whisper = 2001,
    Action = 2002,
    Team = 2003,
    Guild = 2004,
    Spouse = 2006,
    System = 2007,
    Yell = 2008,
    Friend = 2009,
    Center = 2011,
    TopLeft = 2012,
    Ghost = 2013,
    Service = 2014,
    Tip = 2015,
    World = 2021,
    Register = 2100,
    Login = 2101,
    Shop = 2102,
    Vendor = 2104,
    Website = 2105,
    Right1 = 2108,
    Right2 = 2109,
    Offline = 2110,
    Announce = 2111,
    TradeBoard = 2201,
    FriendBoard = 2202,
    TeamBoard = 2203,
    GuildBoard = 2204,
    OthersBoard = 2205,
    Broadcast = 2500,
    Monster = 2600,
    #[num_enum(default)]
    Unknown,
}
/// Enumeration type for controlling how text is stylized in the client's chat
/// area. By default, text appears and fades overtime. This can be overridden
/// with multiple styles, hard-coded into the client.
#[derive(Copy, Clone, Debug, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum TalkStyle {
    Normal = 0,
    Scroll = 1,
    Flash = 2,
    Blast = 3,
    #[num_enum(default)]
    Unknown,
}

/// Message defining a chat message from one player to the other, or from the
/// system to a player. Used for all chat systems in the game, including
/// messages outside of the game world state, such as during character creation
/// or to tell the client to continue logging in after connect.
#[derive(Debug, Default, Deserialize, Serialize, PacketID, Clone)]
#[packet(id = 1004)]
pub struct MsgTalk {
    color: u32,
    channel: u16,
    style: u16,
    character_id: u32,
    recipient_mesh: u32,
    sender_mesh: u32,
    list_count: u8,
    sender_name: String,
    recipient_name: String,
    suffix: String,
    message: String,
}

impl MsgTalk {
    pub fn from_system(
        character_id: u32,
        channel: TalkChannel,
        message: String,
    ) -> Self {
        MsgTalk {
            color: 0x00FF_FFFF,
            channel: channel.into(),
            style: TalkStyle::Normal.into(),
            character_id,
            recipient_mesh: 0,
            sender_mesh: 0,
            list_count: 4,
            sender_name: SYSTEM.to_string(),
            recipient_name: ALL_USERS.to_string(),
            suffix: String::new(),
            message,
        }
    }

    pub fn login_invalid() -> Self {
        Self::from_system(0, TalkChannel::Login, String::from("Login Invalid"))
    }

    pub fn register_invalid() -> Self {
        Self::from_system(
            0,
            TalkChannel::Register,
            String::from("Register Invalid"),
        )
    }

    pub fn register_ok() -> Self {
        Self::from_system(
            0,
            TalkChannel::Register,
            crate::constants::ANSWER_OK.to_owned(),
        )
    }

    pub fn register_name_taken() -> Self {
        Self::from_system(
            0,
            TalkChannel::Register,
            String::from("Character name taken, try another one."),
        )
    }

    pub fn login_ok() -> Self {
        Self::from_system(
            0,
            TalkChannel::Login,
            crate::constants::ANSWER_OK.to_owned(),
        )
    }

    pub fn login_new_role() -> Self {
        Self::from_system(
            0,
            TalkChannel::Login,
            crate::constants::NEW_ROLE.to_owned(),
        )
    }

    pub fn unknown_cmd(&self, message: String) -> Self {
        Self::from_system(self.character_id, TalkChannel::TopLeft, message)
    }
}

#[async_trait]
impl PacketProcess for MsgTalk {
    type ActorState = ActorState;
    type Error = crate::Error;

    async fn process(
        &self,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        if self.message.starts_with('$') {
            // Command Message.
            let (_, command) = self.message.split_at(1);
            let mut parts = command.split_whitespace();
            let command = parts.next().unwrap_or_default();
            match command {
                "dc" => {
                    actor.shutdown().await?;
                },
                "tele" => {
                    let me = actor.character().await?;
                    let map_id = parts
                        .next()
                        .map(|v| v.parse::<u32>())
                        .ok_or_else(|| {
                            self.unknown_cmd(String::from("Bad MapId"))
                                .error_packet()
                        })??;

                    let x = parts
                        .next()
                        .map(|v| v.parse::<u16>())
                        .ok_or_else(|| {
                            self.unknown_cmd(String::from("Bad X"))
                                .error_packet()
                        })??;

                    let y = parts
                        .next()
                        .map(|v| v.parse::<u16>())
                        .ok_or_else(|| {
                            self.unknown_cmd(String::from("Bad Y"))
                                .error_packet()
                        })??;
                    me.teleport(map_id, (x, y)).await?;
                },
                missing => {
                    warn!("Unknown Command {}", missing);
                    let p = self
                        .unknown_cmd(format!("Unkonwn Command {}", missing));
                    actor.send(p).await?;
                },
            };
        }

        Ok(())
    }
}
