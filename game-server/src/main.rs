use async_ctrlc::CtrlC;
use async_trait::async_trait;
use bytes::Bytes;
use futures_util::FutureExt;
use network::{Actor, PacketDecode, PacketHandler, PacketProcess, Server};
use tracing::{debug, info, warn};

mod constants;
mod errors;
mod utils;
use errors::Error;

mod packets;
use packets::{MsgAction, MsgConnect, MsgItem, MsgTalk, PacketType};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct GameServer;

impl Server for GameServer {}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct Handler;

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
            PacketType::MsgConnect => {
                let msg = MsgConnect::decode(&bytes)?;
                debug!("{:?}", msg);
                msg.process(actor).await?;
            },
            PacketType::MsgTalk => {
                let msg = MsgTalk::decode(&bytes)?;
                debug!("{:?}", msg);
                msg.process(actor).await?;
            },
            PacketType::MsgAction => {
                let msg = MsgAction::decode(&bytes)?;
                debug!("{:?}", msg);
                msg.process(actor).await?;
            },
            PacketType::MsgItem => {
                let msg = MsgItem::decode(&bytes)?;
                debug!("{:?}", msg);
                msg.process(actor).await?;
            },
            _ => {
                warn!("{:?}", id);
                return Ok(());
            },
        };
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::init();
    println!(
        r#"
 _____         _____                  
/  __ \       |  ___|                 
| /  \/  ___  | |__  _ __ ___   _   _ 
| |     / _ \ |  __|| '_ ` _ \ | | | |
| \__/\| (_) || |___| | | | | || |_| |
 \____/ \___/ \____/|_| |_| |_| \__,_|
                                      
                                       
Copyright 2020 Shady Khalifa (@shekohex)
     All Rights Reserved.
 "#
    );
    info!("Starting Game Server");
    info!("Initializing server...");

    smol::block_on(async {
        let ctrlc = CtrlC::new()?.map(Ok);
        let server = GameServer::run("0.0.0.0:5817", Handler::default());
        info!("Starting Server on 5817");
        smol::future::race(ctrlc, server).await?;
        Result::<(), Error>::Ok(())
    })?;
    Ok(())
}
