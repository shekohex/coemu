use crate::{Error, PacketEncode};
use async_trait::async_trait;
use bytes::Bytes;
use core::hash::Hash;
use core::ops::Deref;
use core::sync::atomic::{AtomicUsize, Ordering};
use futures::TryFutureExt;
use tokio::sync::mpsc::Sender;
use tracing::instrument;

#[cfg(not(feature = "std"))]
use alloc::{boxed::Box, sync::Arc};
#[cfg(feature = "std")]
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    GenerateKeys(u64),
    Packet(u16, Bytes),
    Shutdown,
}

/// This struct is the main actor type for the server. It is a wrapper around
/// connections to client and its state.
#[derive(Debug)]
pub struct Actor<S: ActorState> {
    handle: ActorHandle,
    state: S,
}

/// A Cheap to clone actor handle. This is used to send messages to the actor
/// from other threads.
///
/// Think of this as a cheap clone of the actor without the state.
#[derive(Clone, Debug)]
pub struct ActorHandle {
    id: Arc<AtomicUsize>,
    tx: Sender<Message>,
}

impl<S: ActorState> Hash for Actor<S> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.handle.id.load(Ordering::Relaxed).hash(state);
    }
}

impl<S: ActorState> PartialEq for Actor<S> {
    fn eq(&self, other: &Self) -> bool {
        self.handle.id.load(Ordering::Relaxed) == other.handle.id.load(Ordering::Relaxed)
    }
}

impl<S: ActorState> Eq for Actor<S> {}

impl<S: ActorState> Deref for Actor<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl From<(u16, Bytes)> for Message {
    fn from((id, bytes): (u16, Bytes)) -> Self {
        Self::Packet(id, bytes)
    }
}

#[async_trait]
pub trait ActorState: Send + Sync + Sized {
    fn init() -> Self;
    /// A good chance to dispose the state and clear anything.
    #[instrument(skip_all, err)]
    async fn dispose(&self, handle: ActorHandle) -> Result<(), Error> {
        tracing::debug!(actor_id = %handle.id(), "Disposing Actor State");
        Ok(())
    }
}

impl ActorState for () {
    fn init() -> Self {}
}

impl<S: ActorState> Actor<S> {
    pub fn new(tx: Sender<Message>) -> Self {
        Self {
            state: S::init(),
            handle: ActorHandle {
                id: Arc::new(AtomicUsize::new(0)),
                tx,
            },
        }
    }

    /// Returns a cheap clone of the actor handle
    pub fn handle(&self) -> ActorHandle {
        self.handle.clone()
    }

    pub fn id(&self) -> usize {
        self.handle.id()
    }

    pub fn set_id(&self, id: usize) {
        self.handle.set_id(id)
    }

    /// Enqueue the packet and send it to the client connected to this actor
    #[instrument(skip(self, packet))]
    pub async fn send<P: PacketEncode>(&self, packet: P) -> Result<(), P::Error> {
        self.handle.send(packet).await
    }

    /// Enqueue the packets and send it all at once to the client connected to
    /// this actor
    #[instrument(skip(self, packets))]
    pub async fn send_all<P, I>(&self, packets: I) -> Result<(), P::Error>
    where
        P: PacketEncode,
        I: IntoIterator<Item = P>,
    {
        self.handle.send_all(packets).await
    }

    #[instrument(skip(self))]
    pub async fn generate_keys(&self, seed: u64) -> Result<(), Error> {
        self.handle.generate_keys(seed).await
    }

    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<(), Error> {
        self.handle.shutdown().await
    }
}

impl ActorHandle {
    pub fn id(&self) -> usize {
        self.id.load(Ordering::Relaxed)
    }

    pub fn set_id(&self, id: usize) {
        self.id.store(id, Ordering::Relaxed);
    }

    /// Enqueue the packet and send it to the client connected to this actor
    #[instrument(skip(self, packet))]
    pub async fn send<P: PacketEncode>(&self, packet: P) -> Result<(), P::Error> {
        let msg = packet.encode()?;
        self.tx.send(msg.into()).map_err(Into::into).await?;
        Ok(())
    }

    /// Enqueue the packets and send it all at once to the client connected to
    /// this actor
    #[instrument(skip(self, packets))]
    pub async fn send_all<P, I>(&self, packets: I) -> Result<(), P::Error>
    where
        P: PacketEncode,
        I: IntoIterator<Item = P>,
    {
        let tasks = packets
            .into_iter()
            .flat_map(|packet| packet.encode().map(|msg| msg.into()))
            .map(|msg| self.tx.send(msg).map_err(crate::Error::from));
        // Wait for all the messages to be sent (in order)
        for task in tasks {
            task.await?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn generate_keys(&self, seed: u64) -> Result<(), Error> {
        let msg = Message::GenerateKeys(seed);
        self.tx.send(msg).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<(), Error> {
        self.tx.send(Message::Shutdown).await?;
        Ok(())
    }
}
