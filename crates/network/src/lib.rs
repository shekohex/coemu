//! This crate contains the core networking components used by the server and
//! client crates.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use bytes::Bytes;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub use async_trait::async_trait;
pub use derive_packethandler::PacketHandler;
pub use derive_packetid::PacketID;
pub use derive_packetprocessor::packet_processor;
pub use tq_codec::TQCodec;
pub use tq_crypto::{CQCipher, Cipher, NopCipher, TQCipher};

mod error;
pub use error::Error;

mod actor;
pub use actor::{Actor, ActorHandle, ActorState, Message};

/// Assoucitates a packet structure with a packet ID. This is used for
/// serialization and deserialization of packets. The packet ID is used to
/// identify the packet type, and the packet structure is used to serialize and
/// deserialize the packet.
pub trait PacketID {
    const PACKET_ID: u16;
}

#[async_trait]
pub trait PacketProcess {
    type Error;
    type ActorState: ActorState;
    type State: Send + Sync;
    /// Process can be invoked by a packet after decode has been called to
    /// structure packet fields and properties. For the server
    /// implementations, this is called in the packet handler after the
    /// message has been dequeued from the server's PacketProcessor
    async fn process(&self, state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error>;
}

pub trait PacketEncode {
    type Error: From<Error> + core::fmt::Debug;
    /// The Packet that we will encode.
    type Packet: Serialize + PacketID;
    /// Encodes the packet structure defined by this message struct into a byte
    /// packet that can be sent to the client. Invoked automatically by the
    /// client's send method. Encodes using byte ordering rules
    /// interoperable with the game client.
    fn encode(&self) -> Result<(u16, Bytes), Self::Error>;
}

pub trait PacketDecode {
    type Error: From<Error> + core::fmt::Debug;
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
    type Error: PacketEncode + Send + Sync;
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
        let id = Self::PACKET_ID;
        let bytes = tq_serde::to_bytes(&self)?;
        Ok((id, bytes.freeze()))
    }
}

impl PacketEncode for (u16, Bytes) {
    type Error = Error;
    type Packet = ();

    fn encode(&self) -> Result<(u16, Bytes), Self::Error> {
        Ok(self.clone())
    }
}

impl<'a> PacketEncode for (u16, &'a [u8]) {
    type Error = Error;
    type Packet = ();

    fn encode(&self) -> Result<(u16, Bytes), Self::Error> {
        let (id, bytes) = self;
        Ok((*id, bytes.to_vec().into()))
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
    const PACKET_ID: u16 = 0;
}

pub struct ErrorPacket<T: PacketEncode>(pub T);

pub trait IntoErrorPacket<T: PacketEncode> {
    fn error_packet(self) -> ErrorPacket<T>;
}

impl<T> IntoErrorPacket<T> for T
where
    T: PacketEncode,
{
    fn error_packet(self) -> ErrorPacket<T> {
        ErrorPacket(self)
    }
}

impl<T> From<T> for ErrorPacket<T>
where
    T: PacketEncode,
{
    fn from(v: T) -> Self {
        Self(v)
    }
}
