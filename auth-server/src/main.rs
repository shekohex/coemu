use async_trait::async_trait;
use bytes::Bytes;
use futures_util::FutureExt;
use network::{Actor, PacketDecode, PacketHandler, PacketProcess, Server};
use tracing::{debug, warn};

mod errors;
use errors::Error;

mod packets;
use async_ctrlc::CtrlC;
use packets::{MsgAccount, PacketType};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct AuthServer;

#[async_trait]
impl Server for AuthServer {}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Handler;

#[async_trait]
impl PacketHandler for Handler {
    type Error = Error;

    async fn handle(
        &self,
        packet: (u16, Bytes),
        actor: &Actor,
    ) -> Result<(), Self::Error> {
        let (id, bytes) = packet;
        match id.into() {
            PacketType::MsgAccount => {
                debug!("Got MsgAccount!");
                let mut msg = MsgAccount::default();
                msg.decode(bytes)?;
                debug!("{:?}", msg);
                msg.process(actor).await?;
                actor.shutdown().await?;
            },
            _ => {
                warn!("Unkown Packet {}", id);
                actor.shutdown().await?;
                return Ok(());
            },
        };
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();

    println!("Starting Auth Server");
    println!("Initializing server...");

    smol::block_on(async {
        let ctrlc = CtrlC::new()?.map(Ok);
        let server = AuthServer::run("0.0.0.0:9958", Handler::default());
        println!("Starting Server on 9958");
        smol::future::race(ctrlc, server).await?;
        Result::<(), Error>::Ok(())
    })?;
    Ok(())
}
