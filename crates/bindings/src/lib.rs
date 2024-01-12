//! Bindings to the host environment.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub use externref::{self as anyref, externref, Resource};

#[externref(crate = "crate::anyref")]
#[link(wasm_import_module = "host")]
extern "C" {
    fn shutdown(actor: &Resource<tq_network::ActorHandle>);
    fn send(
        actor: &Resource<tq_network::ActorHandle>,
        packet_id: u16,
        packet_data_ptr: *const u8,
        packet_data_len: u32,
    );
}

/// Host bindings.
pub mod host {
    use crate::Resource;
    use tq_network::ActorHandle;
    /// Shutdown an actor.
    pub fn shutdown(actor: &Resource<ActorHandle>) {
        unsafe { super::shutdown(actor) }
    }

    /// Send a packet to an actor.
    pub fn send<T: tq_network::PacketEncode>(
        actor: &Resource<ActorHandle>,
        packet: T,
    ) -> Result<(), T::Error> {
        let (packet_id, packet_data) = packet.encode()?;
        unsafe {
            super::send(
                actor,
                packet_id,
                packet_data.as_ptr(),
                packet_data.len() as u32,
            )
        }
        Ok(())
    }
}
