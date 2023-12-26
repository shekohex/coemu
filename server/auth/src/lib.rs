//! Auth Server

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod error;
pub mod state;

pub use error::Error;
pub use state::State;

#[derive(Clone, Copy)]
pub struct Runtime;

impl tq_system::Config for Runtime {
    fn send<P: tq_network::PacketEncode>(
        handle: &tq_system::ActorHandle,
        packet: P,
    ) -> Result<(), P::Error> {
        todo!()
    }

    fn send_all<P, I>(
        handle: &tq_system::ActorHandle,
        packets: I,
    ) -> Result<(), P::Error>
    where
        P: tq_network::PacketEncode,
        I: IntoIterator<Item = P>,
    {
        todo!()
    }

    fn generate_keys(
        handle: &tq_system::ActorHandle,
        seed: u64,
    ) -> Result<(), tq_network::Error> {
        todo!()
    }

    fn shutdown(
        handle: &tq_system::ActorHandle,
    ) -> Result<(), tq_network::Error> {
        todo!()
    }
}

impl msg_account::Config for Runtime {
    type Authanticator = ();
}

impl msg_connect::Config for Runtime {}

impl msg_transfer::Config for Runtime {
    type RealmByName = ();
    type ServerBus = ();
    type TokenGenerator = ();
}
