use crate::{
    packets::{MsgAccount, MsgConnect, PacketType},
    Error, State,
};
use async_trait::async_trait;
use bytes::Bytes;
use network::{Actor, PacketDecode, PacketHandler, PacketProcess};
use tracing::{debug, warn};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Handler;

#[async_trait]
impl PacketHandler for Handler {
    type Error = Error;

    async fn handle(
        &self,
        (id, bytes): (u16, Bytes),
        actor: &Actor,
    ) -> Result<(), Self::Error> {
        let id = id.into();
        match id {
            PacketType::MsgAccount => {
                State::global().add_actor(actor);
                let msg = MsgAccount::decode(&bytes)?;
                debug!("{:?}", msg);
                msg.process(actor).await?;
            },
            PacketType::MsgConnect => {
                let msg = MsgConnect::decode(&bytes)?;
                debug!("{:?}", msg);
                msg.process(actor).await?;
                actor.shutdown().await?;
                State::global().remove_actor(actor);
            },
            _ => {
                warn!("{:?}", id);
                actor.shutdown().await?;
                State::global().remove_actor(actor);
                return Ok(());
            },
        };
        Ok(())
    }
}
