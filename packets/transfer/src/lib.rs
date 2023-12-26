#![cfg_attr(not(feature = "std"), no_std)]

use serde::{Deserialize, Serialize};
use tq_network::{ErrorPacket, PacketEncode, PacketID};

use tq_system::ActorHandle;
pub use traits::{RealmByName, ServerBus, TokenGenerator};

mod functions;
mod traits;
mod types;

/// Defines account parameters to be transferred from the account server to the
/// game server. Account information is supplied from the account database, and
/// used on the game server to transfer authentication and authority level.
#[derive(Clone, Default, Debug, Deserialize, Serialize, PacketID)]
#[packet(id = 4001)]
pub struct MsgTransfer<T: Config> {
    account_id: u32,
    realm_id: u32,
    token: u64,
    #[serde(skip)]
    _config: core::marker::PhantomData<T>,
}

pub trait Config: tq_system::Config {
    /// The type of the authanticator used to authanticate accounts.
    type TokenGenerator: TokenGenerator;
    /// For querying realms by name.
    type RealmByName: RealmByName;
    /// Server bus for checking and transferring accounts
    /// to other servers.
    type ServerBus: ServerBus;
}

impl<T: Config> tq_system::ProcessPacket for MsgTransfer<T> {
    type Error = Error;

    fn process(&self, actor: ActorHandle) -> Result<(), Self::Error> {
        let token = T::TokenGenerator::generate_login_token(
            self.account_id,
            self.realm_id,
        )?;
        let msg = Self {
            account_id: self.account_id,
            realm_id: self.realm_id,
            token,
            _config: core::marker::PhantomData,
        };
        T::send(&actor, msg)?;
        T::shutdown(&actor)?;
        Ok(())
    }
}

/// Possible errors that can occur while processing a packet.
#[derive(Debug, Clone)]
pub enum Error {
    /// Failed to generate a login token.
    TokenGenerationFailed,
    /// The realm is unavailable.
    RealmUnavailable,
    /// Internal Network error.
    Network(tq_network::Error),
    /// An error packet to be sent to the client.
    Msg(u16, bytes::Bytes),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TokenGenerationFailed => {
                write!(f, "Failed to generate a login token")
            },
            Self::RealmUnavailable => write!(f, "Realm is unavailable"),
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::Msg(id, body) => {
                write!(f, "Error packet: id = {}, body = {:?}", id, body)
            },
        }
    }
}

impl From<tq_network::Error> for Error {
    fn from(e: tq_network::Error) -> Self { Self::Network(e) }
}

impl<T: PacketEncode> From<ErrorPacket<T>> for Error {
    fn from(v: ErrorPacket<T>) -> Self {
        let (id, bytes) = v.0.encode().unwrap();
        Self::Msg(id, bytes)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
