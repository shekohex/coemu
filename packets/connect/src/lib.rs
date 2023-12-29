#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::ToString;

cargo_component_bindings::generate!();

pub use bindings::Error;

use bindings::ActorHandle;
use bytes::Bytes;
use serde::Deserialize;
use tq_network::{PacketDecode, PacketID};
use tq_serde::String16;
/// Message containing a connection request to the game server. Contains the
/// player's access token from the Account server, and the patch and language
/// versions of the game client.
#[derive(Debug, Deserialize, PacketID)]
#[packet(id = 1052)]
#[allow(dead_code)]
pub struct MsgConnect {
    id: u32,
    file_contents: u32,
    file_name: String16,
}

impl bindings::Guest for MsgConnect {
    fn process(
        (id, buf): (u16, Vec<u8>),
        actor: &ActorHandle,
    ) -> Result<(), Error> {
        assert_eq!(id, MsgConnect::PACKET_ID, "Invalid packet id");
        let _this = MsgConnect::decode(&Bytes::from(buf))
            .map_err(|e| Error::Decode(e.to_string()))?;
        actor.shutdown();
        Ok(())
    }
}
