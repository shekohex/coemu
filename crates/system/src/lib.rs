//! This crate is used to define common traits for defining packets and systems
//! in the game servers.

#![cfg_attr(not(feature = "std"), no_std)]

use tq_network::ActorHandle;

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

pub trait Config: Send + Sync + 'static {}

#[async_trait::async_trait]
pub trait ProcessPacket {
    type Error;
    async fn process(&self, actor: ActorHandle) -> Result<(), Self::Error>;
}
