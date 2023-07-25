use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error as StdError;

pub use async_trait::async_trait;
pub use derive_packethandler::PacketHandler;
pub use derive_packetid::PacketID;
pub use tq_codec::TQCodec;
pub use tq_crypto::{CQCipher, Cipher, NopCipher, TQCipher};

mod error;
pub use error::Error;

mod actor;
pub use actor::{Actor, ActorHandle, ActorState, Message};

mod server;
pub use server::Server;

pub trait PacketID {
    /// Get the ID of that packet.
    fn id() -> u16;
}

#[async_trait]
pub trait PacketProcess {
    type Error: StdError;
    type ActorState: ActorState;
    type State: Send + Sync;
    /// Process can be invoked by a packet after decode has been called to
    /// structure packet fields and properties. For the server
    /// implementations, this is called in the packet handler after the
    /// message has been dequeued from the server's PacketProcessor
    async fn process(
        &self,
        state: &Self::State,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error>;
}

pub trait PacketEncode {
    type Error: StdError + From<Error>;
    /// The Packet that we will encode.
    type Packet: Serialize + PacketID;
    /// Encodes the packet structure defined by this message struct into a byte
    /// packet that can be sent to the client. Invoked automatically by the
    /// client's send method. Encodes using byte ordering rules
    /// interoperable with the game client.
    fn encode(&self) -> Result<(u16, Bytes), Self::Error>;
}

pub trait PacketDecode {
    type Error: StdError;
    /// The Packet that we will Decode into.
    type Packet: DeserializeOwned;
    /// Decodes a byte packet into the packet structure defined by this message
    /// struct. Should be invoked to structure data from the client for
    /// processing. Decoding follows TQ Digital's byte ordering rules for an
    /// all-binary protocol.
    fn decode(bytes: &Bytes) -> Result<Self::Packet, Self::Error>;
}

#[async_trait]
pub trait PacketHandler {
    type Error: StdError + PacketEncode + Send + Sync;
    type ActorState: ActorState;
    type State: Send + Sync + 'static;
    async fn handle(
        packet: (u16, Bytes),
        state: &Self::State,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error>;
}

impl<T> PacketEncode for T
where
    T: Serialize + PacketID,
{
    type Error = Error;
    type Packet = T;

    fn encode(&self) -> Result<(u16, Bytes), Self::Error> {
        let id = Self::id();
        let bytes = tq_serde::to_bytes(&self)?;
        Ok((id, bytes.freeze()))
    }
}

impl<T> PacketDecode for T
where
    T: DeserializeOwned,
{
    type Error = Error;
    type Packet = T;

    fn decode(bytes: &Bytes) -> Result<T, Self::Error> {
        Ok(tq_serde::from_bytes(bytes)?)
    }
}

impl PacketID for () {
    fn id() -> u16 { 0 }
}

pub struct ErrorPacket<T: PacketEncode>(pub T);

pub trait IntoErrorPacket<T: PacketEncode> {
    fn error_packet(self) -> ErrorPacket<T>;
}

impl<T> IntoErrorPacket<T> for T
where
    T: PacketEncode,
{
    fn error_packet(self) -> ErrorPacket<T> { ErrorPacket(self) }
}

impl<T> From<T> for ErrorPacket<T>
where
    T: PacketEncode,
{
    fn from(v: T) -> Self { Self(v) }
}
