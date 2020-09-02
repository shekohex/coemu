use crate::{Error, PacketEncode};
use bytes::Bytes;
use std::{
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::sync::mpsc::Sender;
use tracing::instrument;

#[derive(Clone, Debug)]
pub enum Message {
    GenerateKeys(u32, u32),
    Packet(u16, Bytes),
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct Actor<S: Send + Sync> {
    id: Arc<AtomicUsize>,
    tx: Sender<Message>,
    state: Arc<S>,
}

impl<S: Send + Sync> Hash for Actor<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.load(Ordering::Relaxed).hash(state);
    }
}

impl<S: Send + Sync> PartialEq for Actor<S> {
    fn eq(&self, other: &Self) -> bool {
        self.id
            .load(Ordering::Relaxed)
            .eq(&other.id.load(Ordering::Relaxed))
    }
}

impl<S: Send + Sync> Eq for Actor<S> {}

impl From<(u16, Bytes)> for Message {
    fn from((id, bytes): (u16, Bytes)) -> Self { Self::Packet(id, bytes) }
}

impl From<(u32, u32)> for Message {
    fn from((key1, key2): (u32, u32)) -> Self { Self::GenerateKeys(key1, key2) }
}

impl<S: Send + Sync> Actor<S> {
    pub fn new(tx: Sender<Message>) -> Self
    where
        S: Default + Send + Sync,
    {
        Self {
            id: Arc::new(AtomicUsize::new(0)),
            state: Arc::new(S::default()),
            tx,
        }
    }

    pub fn id(&self) -> usize { self.id.load(Ordering::Relaxed) }

    pub fn set_id(&self, id: usize) { self.id.store(id, Ordering::Relaxed); }

    pub fn state(&self) -> &S { &self.state }

    /// Enqueue the packet and send it to the client connected to this actor
    pub async fn send<P: PacketEncode>(
        &self,
        packet: P,
    ) -> Result<(), P::Error> {
        let msg = packet.encode()?;
        let mut tx = self.tx.clone();
        tx.send(msg.into()).await.map_err(Into::into)?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn generate_keys(
        &self,
        key1: u32,
        key2: u32,
    ) -> Result<(), Error> {
        let msg = (key1, key2).into();
        let mut tx = self.tx.clone();
        tx.send(msg).await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Error> {
        let mut tx = self.tx.clone();
        tx.send(Message::Shutdown).await?;
        Ok(())
    }
}
