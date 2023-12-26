#![cfg_attr(not(feature = "std"), no_std)]

use serde::Deserialize;
use tq_network::PacketID;
use tq_serde::String16;
use tq_system::ActorHandle;
/// Message containing a connection request to the game server. Contains the
/// player's access token from the Account server, and the patch and language
/// versions of the game client.
#[derive(Debug, Deserialize, PacketID)]
#[packet(id = 1052)]
#[allow(dead_code)]
pub struct MsgConnect<T: Config> {
    id: u32,
    file_contents: u32,
    file_name: String16,
    #[serde(skip)]
    _config: std::marker::PhantomData<T>,
}

pub trait Config: tq_system::Config {}

impl<T: Config> tq_system::ProcessPacket for MsgConnect<T> {
    type Error = Error;

    fn process(&self, actor: ActorHandle) -> Result<(), Self::Error> {
        T::shutdown(&actor)?;
        Ok(())
    }
}

/// Possible errors that can occur while processing a packet.
#[derive(Debug)]
pub enum Error {
    /// Internal Network error.
    Network(tq_network::Error),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl From<tq_network::Error> for Error {
    fn from(e: tq_network::Error) -> Self { Self::Network(e) }
}
