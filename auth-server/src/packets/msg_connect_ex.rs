use super::PacketType;
use network::PacketID;
use serde::Serialize;
use tq_serde::String16;
/// Rejection codes are sent to the client in offset 8 of this packet when the
/// client has failed authentication with the account server. These codes define
/// which error message will be displayed in the client.
#[derive(Debug, PartialEq, Clone)]
pub enum RejectionCode {
    ChangingMap,
    InvalidPassword,
    Ready,
    ServerDown,
    AccountBanned,
    ServerBusy,
    AccountLocked,
    AccountNotActivated,
    AccountActivationFailed,
    ServerTimedOut,
    AccountMaxLoginAttempts,
    ServerLocked,
    ServerOldProtocol,
    Unknown(u32),
}

impl From<RejectionCode> for u32 {
    fn from(original: RejectionCode) -> u32 {
        match original {
            RejectionCode::ChangingMap => 0,
            RejectionCode::InvalidPassword => 1,
            RejectionCode::Ready => 2,
            RejectionCode::ServerDown => 10,
            RejectionCode::AccountBanned => 12,
            RejectionCode::ServerBusy => 20,
            RejectionCode::AccountLocked => 22,
            RejectionCode::AccountNotActivated => 30,
            RejectionCode::AccountActivationFailed => 31,
            RejectionCode::ServerTimedOut => 42,
            RejectionCode::AccountMaxLoginAttempts => 51,
            RejectionCode::ServerLocked => 70,
            RejectionCode::ServerOldProtocol => 73,
            RejectionCode::Unknown(v) => v,
        }
    }
}

impl From<u32> for RejectionCode {
    fn from(original: u32) -> RejectionCode {
        match original {
            0 => RejectionCode::ChangingMap,
            1 => RejectionCode::InvalidPassword,
            2 => RejectionCode::Ready,
            10 => RejectionCode::ServerDown,
            12 => RejectionCode::AccountBanned,
            20 => RejectionCode::ServerBusy,
            22 => RejectionCode::AccountLocked,
            30 => RejectionCode::AccountNotActivated,
            31 => RejectionCode::AccountActivationFailed,
            42 => RejectionCode::ServerTimedOut,
            51 => RejectionCode::AccountMaxLoginAttempts,
            70 => RejectionCode::ServerLocked,
            73 => RejectionCode::ServerOldProtocol,
            val => RejectionCode::Unknown(val),
        }
    }
}

impl Default for RejectionCode {
    fn default() -> Self { RejectionCode::Ready }
}

#[derive(Debug, Default, Serialize)]
pub struct MsgConnectEx {
    authentication_token: u32,
    authentication_code: u32,
    game_server_ip: String16,
    game_server_port: u32,
}

#[derive(Debug, Default, Serialize)]
pub struct MsgConnectRejection {
    reserved: u32,
    rejection_code: u32,
    message: String16,
}

pub struct AccountCredentials {
    pub authentication_token: u32,
    pub authentication_code: u32,
    pub server_ip: String,
    pub server_port: u32,
}

impl MsgConnectEx {
    /// Instantiates a new instance of `MsgConnectRejection` for rejecting a
    /// client connection using a rejection code. The rejection code spawns an
    /// error dialog in the client with a respective error message.
    #[allow(unused)]
    pub fn from_code(code: RejectionCode) -> MsgConnectRejection {
        MsgConnectRejection {
            reserved: 0,
            rejection_code: u32::from(code),
            message: String::new().into(),
        }
    }

    pub fn forword_connection(acc_credentials: AccountCredentials) -> Self {
        MsgConnectEx {
            authentication_token: acc_credentials.authentication_token,
            authentication_code: acc_credentials.authentication_code,
            game_server_ip: acc_credentials.server_ip.into(),
            game_server_port: acc_credentials.server_port,
        }
    }
}

impl PacketID for MsgConnectRejection {
    type ID = PacketType;

    fn id(&self) -> Self::ID { PacketType::MsgConnectEx }
}

impl PacketID for MsgConnectEx {
    type ID = PacketType;

    fn id(&self) -> Self::ID { PacketType::MsgConnectEx }
}
