mod msg_account;
pub use msg_account::MsgAccount;

mod msg_connect_ex;
pub use msg_connect_ex::{AccountCredentials, MsgConnectEx, RejectionCode};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PacketType {
    MsgAccount,
    MsgConnectEx,
    MsgUnKnown(u16),
}

impl From<PacketType> for u16 {
    fn from(original: PacketType) -> u16 {
        use PacketType::*;
        match original {
            MsgAccount => 1051,
            MsgConnectEx => 1055,
            _ => 0,
        }
    }
}
impl From<u16> for PacketType {
    fn from(v: u16) -> Self {
        match v {
            1051 => Self::MsgAccount,
            1055 => Self::MsgConnectEx,
            id => Self::MsgUnKnown(id),
        }
    }
}
