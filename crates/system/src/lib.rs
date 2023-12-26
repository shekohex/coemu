//! This crate is used to define common traits for defining packets and systems
//! in the game servers.

#![cfg_attr(not(feature = "std"), no_std)]

use core::{sync::atomic::AtomicUsize, sync::atomic::Ordering};

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Arc;
use tq_network::PacketEncode;

#[derive(Clone, Debug)]
pub struct ActorHandle {
    id: Arc<AtomicUsize>,
}

impl ActorHandle {
    pub fn id(&self) -> usize { self.id.load(Ordering::Relaxed) }

    pub fn set_id(&self, id: usize) { self.id.store(id, Ordering::Relaxed); }
}

/// A trait for querying a single value from a type.
///
/// It is not required that the value is constant.
pub trait Get<T> {
    /// Return the current value.
    fn get() -> T;
}

impl<T: Default> Get<T> for () {
    fn get() -> T { T::default() }
}

/// Implement Get by returning Default for any type that implements Default.
pub struct GetDefault;
impl<T: Default> Get<T> for GetDefault {
    fn get() -> T { T::default() }
}

pub trait Config: Send + Sync + 'static {
    /// Enqueue the packet and send it to the client connected to this actor
    fn send<P: PacketEncode>(
        handle: &ActorHandle,
        packet: P,
    ) -> Result<(), P::Error>;

    /// Enqueue the packets and send it all at once to the client connected to
    /// this actor
    fn send_all<P, I>(handle: &ActorHandle, packets: I) -> Result<(), P::Error>
    where
        P: PacketEncode,
        I: IntoIterator<Item = P>;

    fn generate_keys(
        handle: &ActorHandle,
        seed: u64,
    ) -> Result<(), tq_network::Error>;

    /// Shutdown the actor and disconnect the client.
    fn shutdown(handle: &ActorHandle) -> Result<(), tq_network::Error>;
}

pub trait ProcessPacket {
    type Error;
    fn process(&self, actor: ActorHandle) -> Result<(), Self::Error>;
}
