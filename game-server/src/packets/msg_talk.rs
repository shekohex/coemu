use super::PacketType;
use crate::constants::{ALL_USERS, SYSTEM};
use async_trait::async_trait;
use network::{Actor, PacketID, PacketProcess};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Enumeration for defining the channel text is printed to. Can also print to
/// separate states of the client such as character registration, and can be
/// used to change the state of the client or deny a login.
pub enum TalkChannel {
    Talk,
    Whisper,
    Action,
    Team,
    Guild,
    Spouse,
    System,
    Yell,
    Friend,
    Center,
    TopLeft,
    Ghost,
    Service,
    Tip,
    World,
    Create,
    Login,
    Shop,
    Vendor,
    Website,
    Right1,
    Right2,
    Offline,
    Announce,
    TradeBoard,
    FriendBoard,
    TeamBoard,
    GuildBoard,
    OthersBoard,
    Broadcast,
    Monster,
    Unknown(u16),
}

impl From<TalkChannel> for u16 {
    fn from(val: TalkChannel) -> u16 {
        match val {
            TalkChannel::Talk => 2000,
            TalkChannel::Whisper => 2001,
            TalkChannel::Action => 2002,
            TalkChannel::Team => 2003,
            TalkChannel::Guild => 2004,
            TalkChannel::Spouse => 2006,
            TalkChannel::System => 2007,
            TalkChannel::Yell => 2008,
            TalkChannel::Friend => 2009,
            TalkChannel::Center => 2011,
            TalkChannel::TopLeft => 2012,
            TalkChannel::Ghost => 2013,
            TalkChannel::Service => 2014,
            TalkChannel::Tip => 2015,
            TalkChannel::World => 2021,
            TalkChannel::Create => 2100,
            TalkChannel::Login => 2101,
            TalkChannel::Shop => 2102,
            TalkChannel::Vendor => 2104,
            TalkChannel::Website => 2105,
            TalkChannel::Right1 => 2108,
            TalkChannel::Right2 => 2109,
            TalkChannel::Offline => 2110,
            TalkChannel::Announce => 2111,
            TalkChannel::TradeBoard => 2201,
            TalkChannel::FriendBoard => 2202,
            TalkChannel::TeamBoard => 2203,
            TalkChannel::GuildBoard => 2204,
            TalkChannel::OthersBoard => 2205,
            TalkChannel::Broadcast => 2500,
            TalkChannel::Monster => 2600,
            _ => 0,
        }
    }
}

impl From<u16> for TalkChannel {
    fn from(val: u16) -> TalkChannel {
        match val {
            2000 => TalkChannel::Talk,
            2001 => TalkChannel::Whisper,
            2002 => TalkChannel::Action,
            2003 => TalkChannel::Team,
            2004 => TalkChannel::Guild,
            2006 => TalkChannel::Spouse,
            2007 => TalkChannel::System,
            2008 => TalkChannel::Yell,
            2009 => TalkChannel::Friend,
            2011 => TalkChannel::Center,
            2012 => TalkChannel::TopLeft,
            2013 => TalkChannel::Ghost,
            2014 => TalkChannel::Service,
            2015 => TalkChannel::Tip,
            2021 => TalkChannel::World,
            2100 => TalkChannel::Create,
            2101 => TalkChannel::Login,
            2102 => TalkChannel::Shop,
            2104 => TalkChannel::Vendor,
            2105 => TalkChannel::Website,
            2108 => TalkChannel::Right1,
            2109 => TalkChannel::Right2,
            2110 => TalkChannel::Offline,
            2111 => TalkChannel::Announce,
            2201 => TalkChannel::TradeBoard,
            2202 => TalkChannel::FriendBoard,
            2203 => TalkChannel::TeamBoard,
            2204 => TalkChannel::GuildBoard,
            2205 => TalkChannel::OthersBoard,
            2500 => TalkChannel::Broadcast,
            2600 => TalkChannel::Monster,
            val => TalkChannel::Unknown(val),
        }
    }
}
/// Enumeration type for controlling how text is stylized in the client's chat
/// area. By default, text appears and fades overtime. This can be overridden
/// with multiple styles, hard-coded into the client.
pub enum TalkStyle {
    Normal,
    Scroll,
    Flash,
    Blast,
    Unknown(u16),
}

impl From<TalkStyle> for u16 {
    fn from(val: TalkStyle) -> u16 {
        match val {
            TalkStyle::Normal => 0,
            TalkStyle::Scroll => 1,
            TalkStyle::Flash => 2,
            TalkStyle::Blast => 4,
            _ => 0,
        }
    }
}

impl From<u16> for TalkStyle {
    fn from(val: u16) -> TalkStyle {
        match val {
            0 => TalkStyle::Normal,
            1 => TalkStyle::Scroll,
            2 => TalkStyle::Flash,
            4 => TalkStyle::Blast,
            val => TalkStyle::Unknown(val),
        }
    }
}

/// Message defining a chat message from one player to the other, or from the
/// system to a player. Used for all chat systems in the game, including
/// messages outside of the game world state, such as during character creation
/// or to tell the client to continue logging in after connect.
#[derive(Debug, Default, Deserialize, Serialize)]
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
            channel: u16::from(channel),
            style: u16::from(TalkStyle::Normal),
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
}

impl PacketID for MsgTalk {
    type ID = PacketType;

    fn id(&self) -> Self::ID { PacketType::MsgTalk }
}

#[async_trait]
impl PacketProcess for MsgTalk {
    type Error = crate::Error;

    async fn process(&self, actor: &Actor) -> Result<(), Self::Error> {
        if self.message.starts_with('$') {
            // Command Message.
            let (_, command) = self.message.split_at(1);
            match command {
                "dc" => {
                    actor.shutdown().await?;
                },
                missing => {
                    warn!("Unknown Command {}", missing);
                    let p = MsgTalk::from_system(
                        1,
                        TalkChannel::TopLeft,
                        format!("Unkonwn Command {}", missing),
                    );
                    debug!("{:?}", p);
                    actor.send(p).await?;
                },
            };
        }

        Ok(())
    }
}
