#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

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

pub enum Error {
    Network(tq_network::Error),
}

impl From<tq_network::Error> for Error {
    fn from(v: tq_network::Error) -> Self { Self::Network(v) }
}

#[tq_network::packet_processor(MsgConnect)]
fn process(
    msg: MsgConnect,
    actor: &Resource<ActorHandle>,
) -> Result<(), crate::Error> {
    host::log(tracing::Level::DEBUG, "msgconnect", "hello?");
    assert!(tracing::dispatcher::get_default(|_| Some(())).is_some());
    tracing::debug!(target: "msgconnect", ?msg, "Shutting down actor");
    host::send(actor, msg)?;
    host::shutdown(actor);
    Ok(())
}
