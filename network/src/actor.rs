use crate::{Error, PacketEncode};
use bytes::Bytes;
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
    tx: Sender<Message>,
}

impl From<(u16, Bytes)> for Message {
    fn from((id, bytes): (u16, Bytes)) -> Self { Self::Packet(id, bytes) }
}

impl From<(u32, u32)> for Message {
    fn from((key1, key2): (u32, u32)) -> Self { Self::GenerateKeys(key1, key2) }
}

impl Actor {
    pub fn new(tx: Sender<Message>) -> Self { Self { tx } }

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
