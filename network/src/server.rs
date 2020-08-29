use crate::{actor::Message, Actor, Error, PacketHandler};
use async_trait::async_trait;
use crypto::Cipher;
use std::fmt::Debug;
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    stream::StreamExt,
    sync::mpsc,
};
use tq_codec::{TQCodec, TQEncoder};
use tracing::{debug, error, instrument};

#[async_trait]
pub trait Server {
    type Cipher: Cipher;
    type PacketHandler: PacketHandler;

    #[instrument]
    async fn run<A>(addr: A) -> Result<(), Error>
    where
        A: Debug + ToSocketAddrs + Send + Sync,
    {
        run::<Self::PacketHandler, A, Self::Cipher>(addr).await
    }
}

#[instrument()]
async fn run<H, A, C>(addr: A) -> Result<(), Error>
where
    H: PacketHandler,
    A: Debug + ToSocketAddrs + Send + Sync,
    C: Cipher,
{
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
        let task = async move {
            if let Err(e) = handle_stream::<H, C>(stream).await {
                error!("{}", e);
            }
            debug!("Task Ended.");
        };
        tokio::spawn(task);
    }
    Ok(())
}
#[instrument(skip(stream))]
async fn handle_stream<H, C>(stream: TcpStream) -> Result<(), Error>
where
    H: PacketHandler,
    C: Cipher,
{
    let (tx, rx) = mpsc::channel(50);
    let actor = Actor::new(tx);
    let cipher = C::default();
    let (encoder, mut decoder) = TQCodec::new(stream, cipher.clone()).split();
    // Start MsgHandler in a seprate task.
    tokio::spawn(handle_msg(rx, encoder, cipher));

    while let Some(packet) = decoder.next().await {
        let (id, bytes) = packet?;
        H::handle((id, bytes), &actor)
            .await
            .map_err(|e| Error::Other(e.to_string()))?;
    }
    debug!("Socket Closed, stopping task.");
    Ok(())
}

#[instrument(skip(rx, encoder, cipher))]
async fn handle_msg<C: Cipher>(
    mut rx: mpsc::Receiver<Message>,
    mut encoder: TQEncoder<TcpStream, C>,
    cipher: C,
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
