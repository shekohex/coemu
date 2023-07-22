use crate::{Error, PacketEncode};
use async_trait::async_trait;
use bytes::Bytes;
use futures::TryFutureExt;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tracing::instrument;

#[derive(Clone, Debug)]
pub enum Message {
    GenerateKeys(u64),
    Packet(u16, Bytes),
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct Actor<S: ActorState> {
    id: Arc<AtomicUsize>,
    tx: Sender<Message>,
    state: S,
}

/// Default actor comes with Empty State.
impl<S: ActorState> Default for Actor<S> {
    fn default() -> Self {
        let (tx, _) = mpsc::channel(1);
        Self {
            id: Default::default(),
            state: S::empty(),
            tx,
        }
    }
}

impl<S: ActorState> Hash for Actor<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.load(Ordering::Relaxed).hash(state);
    }
}

impl<S: ActorState> PartialEq for Actor<S> {
    fn eq(&self, other: &Self) -> bool {
        self.id
            .load(Ordering::Relaxed)
            .eq(&other.id.load(Ordering::Relaxed))
    }
}

impl<S: ActorState> Eq for Actor<S> {}

impl<S: ActorState> Deref for Actor<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target { &self.state }
}

impl From<(u16, Bytes)> for Message {
    fn from((id, bytes): (u16, Bytes)) -> Self { Self::Packet(id, bytes) }
}

#[async_trait]
pub trait ActorState: Send + Sync + Sized + Clone {
    fn init() -> Self;
    /// Should be only used for the Default Actor
    fn empty() -> Self;
    /// A good chance to dispose the state and clear anything.
    async fn dispose(&self, actor: &Actor<Self>) -> Result<(), Error> {
        let _ = actor;
        Ok(())
    }
}

impl ActorState for () {
    fn init() -> Self {}

    fn empty() -> Self {}
}

impl<S: ActorState> Actor<S> {
    pub fn new(tx: Sender<Message>) -> Self {
        Self {
            id: Arc::new(AtomicUsize::new(0)),
            state: S::init(),
            tx,
        }
    }

    pub fn id(&self) -> usize { self.id.load(Ordering::Relaxed) }

    pub fn set_id(&self, id: usize) { self.id.store(id, Ordering::Relaxed); }

    /// Enqueue the packet and send it to the client connected to this actor
    pub async fn send<P: PacketEncode>(
        &self,
        packet: P,
    ) -> Result<(), P::Error> {
        let msg = packet.encode()?;
        self.tx.send(msg.into()).map_err(Into::into).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn generate_keys(&self, seed: u64) -> Result<(), Error> {
        let msg = Message::GenerateKeys(seed);
        self.tx.send(msg).await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Error> {
        self.tx.send(Message::Shutdown).await?;
        Ok(())
    }
}
