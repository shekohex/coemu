use crate::{actor::Message, Actor, Error, PacketHandler};
use async_trait::async_trait;
use crypto::{Cipher, TQCipher};
use std::net::SocketAddr;
use tokio::{
    net::{TcpListener, TcpStream},
    stream::StreamExt,
    sync::mpsc,
};
use tq_codec::{TQCodec, TQEncoder};
use tracing::{debug, error, instrument};

#[async_trait]
pub trait Server {
    #[instrument(skip(self, handler))]
    async fn run<H: PacketHandler>(
        &self,
        addr: &str,
        handler: H,
    ) -> Result<(), Error> {
        let addr: SocketAddr = addr.parse()?;
        let mut listener = TcpListener::bind(addr).await?;
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream?;
            debug!("Got Connection from {}", stream.peer_addr()?);
            stream.set_nodelay(true)?;
            stream.set_linger(None)?;
            stream.set_recv_buffer_size(64)?;
            stream.set_send_buffer_size(64)?;
            stream.set_ttl(5)?;
            let handler = handler.clone();
            let task = async move {
                if let Err(e) = handle_stream(stream, handler).await {
                    error!("{}", e);
                }
                debug!("Task Ended.");
            };
            tokio::spawn(task);
        }
        Ok(())
    }
}

#[instrument(skip(handler, stream))]
async fn handle_stream<H: PacketHandler>(
    stream: TcpStream,
    handler: H,
) -> Result<(), Error> {
    let (tx, rx) = mpsc::channel(50);
    let actor = Actor::new(tx);
    let cipher = TQCipher::new();
    let (encoder, mut decoder) = TQCodec::new(stream, cipher.clone()).split();
    // Start MsgHandler in a seprate task.
    tokio::spawn(handle_msg(rx, encoder, cipher));

    while let Some(packet) = decoder.next().await {
        let (id, bytes) = packet?;
        handler
            .handle((id, bytes), &actor)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
    }
    debug!("Socket Closed, stopping task.");
    Ok(())
}

#[instrument(skip(rx, encoder, cipher))]
async fn handle_msg(
    mut rx: mpsc::Receiver<Message>,
    mut encoder: TQEncoder<TcpStream, TQCipher>,
    cipher: impl Cipher,
) -> Result<(), Error> {
    use Message::*;
    while let Some(msg) = rx.next().await {
        match msg {
            GenerateKeys(key1, key2) => {
                cipher.generate_keys(key1, key2);
            },
            Packet(id, bytes) => {
                encoder.send((id, bytes)).await?;
            },
            Shutdown => {
                encoder.close().await?;
                break;
            },
        };
    }
    debug!("Socket Closed, stopping handle message.");
    Ok(())
}
