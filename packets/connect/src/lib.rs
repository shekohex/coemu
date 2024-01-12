#![no_std]

extern crate alloc;

use tq_bindings::Resource;
use tq_network::ActorHandle;
use tq_serde::String16;

tq_bindings::generate!();


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

#[repr(C)]
pub enum Error {
    Network(tq_network::Error),
}

#[tq_network::packet_processor(MsgConnect)]
fn process(
    _msg: MsgConnect,
    actor: &Resource<ActorHandle>,
) -> Result<(), crate::Error> {
    host::shutdown(actor);
    Ok(())
}
