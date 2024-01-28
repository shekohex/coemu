#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

include!(concat!(env!("OUT_DIR"), "/wasm.rs"));

use tq_bindings::{host, Resource};
use tq_network::ActorHandle;
use tq_serde::String16;

/// Message containing a connection request to the game server. Contains the
/// player's access token from the Account server, and the patch and language
/// versions of the game client.
#[derive(Debug, serde::Serialize, serde::Deserialize, tq_network::PacketID)]
#[packet(id = 1052)]
pub struct MsgConnect {
    pub id: u32,
    pub file_contents: u32,
    pub file_name: String16,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] tq_network::Error),
}

#[tq_network::packet_processor(MsgConnect)]
pub fn process(msg: MsgConnect, actor: &Resource<ActorHandle>) -> Result<(), crate::Error> {
    tracing::debug!(?msg, "Shutting down actor!");
    host::network::actor::shutdown(actor);
    Ok(())
}
