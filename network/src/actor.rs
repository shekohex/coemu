use crate::{Error, PacketEncode};
use bytes::Bytes;
use std::hash::Hash;
use tokio::sync::mpsc::Sender;
use tracing::instrument;

#[derive(Clone, Debug)]
pub enum Message {
    GenerateKeys(u32, u32),
    Packet(u16, Bytes),
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct Actor {
    id: usize,
    tx: Sender<Message>,
}

impl Hash for Actor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}

impl PartialEq for Actor {
    fn eq(&self, other: &Self) -> bool { self.id.eq(&other.id) }
}

impl Eq for Actor {}

impl From<(u16, Bytes)> for Message {
    fn from((id, bytes): (u16, Bytes)) -> Self { Self::Packet(id, bytes) }
}

impl From<(u32, u32)> for Message {
    fn from((key1, key2): (u32, u32)) -> Self { Self::GenerateKeys(key1, key2) }
}

impl Actor {
    pub fn new(tx: Sender<Message>) -> Self {
        Self {
            id: fastrand::usize(0..usize::MAX),
            tx,
        }
    }

    pub fn id(&self) -> usize { self.id }

    /// Enqueue the packet and send it to the client connected to this actor
    pub async fn send(&self, packet: impl PacketEncode) -> Result<(), Error> {
        let msg = packet.encode()?;
        let mut tx = self.tx.clone();
        tx.send(msg.into()).await?;
        Ok(())
    }

    #[instrument]
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
