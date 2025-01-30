#![cfg_attr(not(feature = "std"), no_std)]

use num_enum::IntoPrimitive;
use serde::Serialize;
use tq_network::PacketID;
use tq_serde::String16;

/// Rejection codes are sent to the client in offset 8 of this packet when the
/// client has failed authentication with the account server. These codes define
/// which error message will be displayed in the client.
#[derive(Debug, IntoPrimitive, Copy, Clone)]
#[repr(u32)]
pub enum RejectionCode {
    Clear = 0,
    InvalidPassword = 1,
    Ready = 2,
    ServerDown = 10,
    TryAgainLater = 11,
    AccountBanned = 12,
    ServerBusy = 20,
    AccountLocked = 22,
    AccountNotActivated = 30,
    AccountActivationFailed = 31,
    ServerTimedOut = 42,
    AccountMaxLoginAttempts = 51,
    ServerLocked = 70,
    ServerOldProtocol = 73,
}

impl RejectionCode {
    pub fn packet(self) -> MsgConnectRejection {
        MsgConnectEx::from_code(self)
    }
}

#[derive(Debug, Serialize, PacketID)]
#[packet(id = 1055)]
pub struct MsgConnectEx {
    token: u64,
    game_server_ip: String16,
    game_server_port: u32,
}

#[derive(Debug, Serialize, PacketID)]
#[packet(id = 1055)]
pub struct MsgConnectRejection {
    reserved: u32,
    rejection_code: u32,
    message: String16,
}

#[derive(Debug, Clone)]
pub struct AccountCredentials {
    pub token: u64,
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
            rejection_code: code.into(),
            message: String::new().into(),
        }
    }

    pub fn forword_connection(acc_credentials: AccountCredentials) -> Self {
        MsgConnectEx {
            token: acc_credentials.token,
            game_server_ip: acc_credentials.server_ip.into(),
            game_server_port: acc_credentials.server_port,
        }
    }
}
