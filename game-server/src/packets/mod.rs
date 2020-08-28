mod msg_connect;
pub use msg_connect::MsgConnect;

mod msg_talk;
pub use msg_talk::{MsgTalk, TalkChannel, TalkStyle};

mod msg_user_info;
pub use msg_user_info::MsgUserInfo;

mod msg_action;
pub use msg_action::MsgAction;

mod msg_item;
pub use msg_item::MsgItem;

/// Packet types for the Conquer Online game client across all server projects.
/// Identifies packets by an unsigned short from offset 2 of every packet sent
/// to the server.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PacketType {
    MsgRegister,
    MsgTalk,
    MsgUserInfo,
    MsgItem,
    MsgAction,
    MsgConnect,
    MsgUnKnown(u16),
}

impl From<u16> for PacketType {
    fn from(original: u16) -> PacketType {
        match original {
            1001 => Self::MsgRegister,
            1004 => Self::MsgTalk,
            1006 => Self::MsgUserInfo,
            1009 => Self::MsgItem,
            1010 => Self::MsgAction,
            1052 => Self::MsgConnect,
            id => Self::MsgUnKnown(id),
        }
    }
}

impl From<PacketType> for u16 {
    fn from(original: PacketType) -> u16 {
        use PacketType::*;
        match original {
            MsgRegister => 1001,
            MsgTalk => 1004,
            MsgUserInfo => 1006,
            MsgItem => 1009,
            MsgAction => 1010,
            MsgConnect => 1052,
            _ => 0,
        }
    }
}
