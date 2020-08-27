use crate::{actor::Message, Actor, Error, PacketHandler};
use async_trait::async_trait;
use bytes::Bytes;
use std::net::SocketAddr;
use tokio::{
    net::{TcpListener, TcpStream},
    stream::StreamExt,
    sync::mpsc,
};
use tq_codec::TQCodec;
use tracing::{debug, error, instrument};

#[async_trait]
pub trait Server {
    #[instrument(skip(handler))]
    async fn run(addr: &str, handler: impl PacketHandler) -> Result<(), Error> {
        let addr: SocketAddr = addr.parse()?;
        let mut listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            debug!("Got Connection from {}", stream.peer_addr()?);
            stream.set_nodelay(true)?;
            let handler = handler.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_stream(stream, handler).await {
                    error!("Error For Stream: {}", e);
                }
            });
        }
        Ok(())
    }
}

/// Represents what happened "next" that we should handle.
#[derive(Debug)]
enum SelectResult {
    Packet(u16, Bytes),
    Command(Message),
}

#[instrument(skip(handler, stream))]
async fn handle_stream(
    stream: TcpStream,
    handler: impl PacketHandler,
) -> Result<(), Error> {
    let (tx, mut rx) = mpsc::channel(5);
    let mut codec = TQCodec::new(stream);
    let actor = Actor::new(tx);
    loop {
        let msg_fut = codec.next();
        let cmd_fut = rx.next();
        let sel_res = tokio::select! {
            msg = msg_fut => {
                let (id, bytes) = msg??;
                SelectResult::Packet(id, bytes)
            },
            cmd = cmd_fut => SelectResult::Command(cmd?)
        };
        use SelectResult::*;
        match sel_res {
            Packet(id, bytes) => {
                if let Err(e) = handler.handle((id, bytes), &actor).await {
                    error!("Error While Handling Packet {} {}", id, e);
                    break;
                }
            },
            Command(cmd) => {
                use Message::*;
                match cmd {
                    GenerateKeys(key1, key2) => {
                        codec.generate_keys(key1, key2);
                    },
                    Packet(id, bytes) => {
                        codec.send((id, bytes)).await?;
                    },
                    Shutdown => {
                        codec.close().await?;
                        break;
                    },
                };
            },
        }
    }
    Ok(())
}
