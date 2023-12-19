#![cfg_attr(not(feature = "std"), no_std)]

use msg_connect_ex::{MsgConnectEx, RejectionCode};
use msg_transfer::MsgTransfer;
use serde::Deserialize;
use tq_network::{ActorHandle, PacketID};
use tq_serde::{String16, TQPassword};

pub use traits::Authanticator;

mod functions;
mod traits;
mod types;

#[derive(Debug, Deserialize, PacketID)]
#[packet(id = 1051)]
pub struct MsgAccount<T: Config> {
    pub username: String16,
    pub password: TQPassword,
    pub realm: String16,
    #[serde(skip)]
    pub rejection_code: u32,
    #[serde(skip)]
    pub account_id: i32,
    #[serde(skip)]
    _config: core::marker::PhantomData<T>,
}

pub trait Config: msg_transfer::Config {
    /// The type of the authanticator used to authanticate accounts.
    type Authanticator: Authanticator;
}

#[async_trait::async_trait]
impl<T: Config> tq_system::ProcessPacket for MsgAccount<T> {
    type Error = Error;

    async fn process(&self, actor: ActorHandle) -> Result<(), Self::Error> {
        let maybe_accont_id =
            T::Authanticator::auth(&self.username, &self.password).await;
        let account_id = match maybe_accont_id {
            Ok(id) => id,
            Err(e) => {
                let res = match e {
                    Error::InvalidUsernameOrPassword => {
                        RejectionCode::InvalidPassword.packet()
                    },
                    _ => {
                        tracing::error!("Error authenticating account: {e}");
                        RejectionCode::TryAgainLater.packet()
                    },
                };
                actor.send(res).await?;
                return Ok(());
            },
        };
        actor.set_id(account_id as usize);
        let res = match MsgTransfer::<T>::handle(&actor, &self.realm).await {
            Ok(res) => res,
            _ => {
                tracing::warn!(
                    %account_id,
                    "Failed to transfer account"
                );
                return Ok(());
            },
        };
        let res = MsgConnectEx::forword_connection(res);
        actor.send(res).await?;
        Ok(())
    }
}

/// Possible errors that can occur while processing a packet.
pub enum Error {
    /// User has entered an invalid username or password.
    InvalidUsernameOrPassword,
    /// Internal Network error.
    Network(tq_network::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidUsernameOrPassword => {
                write!(f, "Invalid username or password")
            },
            Self::Network(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl From<tq_network::Error> for Error {
    fn from(e: tq_network::Error) -> Self { Self::Network(e) }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {}
}
