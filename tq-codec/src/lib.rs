//! crate that contians a [`TQCodec`] that wraps any [`AsyncRead`] +
//! [`AsyncWrite`] and Outputs a [`Stream`] of `(u16, Bytes)` where the `u16` is
//! the PacketID and Bytes is the Body of the Packet.
//! It also implements [`Sink`] where you could write `(u16, Bytes)` to it.
//!
//! Basiclly, Client Packets are length-prefixed bytes, where the First 2
//! bytes are the length of the packet, next 2 bytes are the ID of the packet,
//! and after that are the body of that packets.
//! Here a simple ASCII Art to imagine how that looks like in memory.
//!
//! ```text
//! +----+----+----+----+----+----+----+----+----
//! | 00 | 12 | 00 | 15 | 01 | 12 | 14 | 11 | 02 ...
//! +----+----+----+----+----+----+----+----+----
//! ^^^^^^^^^^  ^^^^^^^^
//! |           |
//! the first 2 | bytes are the length
//!             the next 2 bytes are the packet id.
//! ```

use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::future::Future;
use crypto::{Cipher, TQCipher};
use pretty_hex::PrettyHex;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    stream::Stream,
};
use tracing::{debug, warn};

/// A simple State Machine for Decoding the Stream.
#[derive(Debug, Clone, Copy)]
enum DecodeState {
    /// Decoding the Head of the Packet (4 Bytes for the Size + PacketID)
    Head,
    /// Decoding the Packet Bytes (usize => Packet Size, u16 => PacketID)
    Data((usize, u16)),
}

#[derive(Debug)]
pub struct TQCodec<S: AsyncRead + AsyncWrite> {
    /// Current Decode State
    state: DecodeState,
    /// Cipher Used to Decrypt/Encrypt Packets
    cipher: TQCipher,
    /// The Underlaying Stream (like a TcpStream).
    stream: S,
    /// Buffer used when reading from the stream. Data is not returned from
    /// this buffer until an entire packet has been read.
    rd: BytesMut,
    /// Buffer used to stage data before writing it to the socket.
    wr: BytesMut,
}

impl<S: AsyncRead + AsyncWrite + Unpin> TQCodec<S> {
    pub fn new(stream: S) -> Self {
        Self {
            state: DecodeState::Head,
            cipher: TQCipher::default(),
            stream,
            wr: BytesMut::with_capacity(64),
            rd: BytesMut::with_capacity(64),
        }
    }

    pub fn generate_keys(&mut self, key1: u32, key2: u32) {
        self.cipher.generate_keys(key1, key2);
    }

    pub async fn send(&mut self, item: (u16, Bytes)) -> Result<(), io::Error> {
        let buf = self.encode_data(item.0, item.1)?;
        self.buffer(&buf);
        self.flush().await?;
        Ok(())
    }

    pub async fn close(&mut self) -> Result<(), io::Error> {
        self.stream.shutdown().await?;
        Ok(())
    }

    /// Read data from the socket.
    ///
    /// This only returns `Ready` when the socket has closed.
    async fn fill_read_buf(&mut self) -> Result<(), io::Error> {
        self.rd.reserve(64);
        // Read data into the buffer.
        let n = self.stream.read_buf(&mut self.rd).await?;
        if n == 0 {
            Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Read Zero"))
        } else {
            Ok(())
        }
    }

    /// Flush the write buffer to the socket
    async fn flush(&mut self) -> Result<(), io::Error> {
        // As long as there is buffered data to write, try to write it.
        self.stream.write_all(&self.wr).await?;
        // This discards the first `n` bytes of the buffer.
        let _ = self.wr.split_to(self.wr.len());
        Ok(())
    }

    /// Buffer a packet.
    ///
    /// This writes the packet to an internal buffer. Calls to `poll_flush` will
    /// attempt to flush this buffer to the socket.
    fn buffer(&mut self, buf: &[u8]) {
        // Ensure the buffer has capacity. Ideally this would not be unbounded,
        // but to keep the example simple, we will not limit this.
        self.wr.reserve(64);

        // Push the packet onto the end of the write buffer.
        //
        // The `put` function is from the `BufMut` trait.
        self.wr.put(buf);
    }

    fn decode_head(&mut self) -> io::Result<Option<(usize, u16)>> {
        if self.rd.len() < 4 {
            // Not enough data
            return Ok(None);
        }
        let (n, packet_type) = {
            let mut len = [0u8; 2];
            let mut ty = [0u8; 2];
            // Get the decrypted head len.
            self.cipher.decrypt(&self.rd[0..2], &mut len);
            // Get the decrypted head packet type.
            self.cipher.decrypt(&self.rd[2..4], &mut ty);
            // get length
            let n = len.as_ref().get_u16_le();
            // get type
            let packet_id = ty.as_ref().get_u16_le();
            if n > 8_000 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Frame Too Big",
                ));
            }
            (n as usize, packet_id)
        };
        // Ensure that the buffer has enough space to read the incoming
        // payload
        self.rd.reserve(n - 4);
        // Drop the header
        let _ = self.rd.split_to(4);

        Ok(Some((n - 4, packet_type)))
    }

    fn decode_data(&mut self, n: usize) -> io::Result<Option<BytesMut>> {
        // At this point, the buffer has already had the required capacity
        // reserved. All there is to do is read.

        if self.rd.len() < n {
            return Ok(None);
        }
        let buf = &mut self.rd.split_to(n);
        let mut data = BytesMut::with_capacity(n);
        data.resize(n, 0);
        // Decrypt the data
        self.cipher.decrypt(&buf, &mut data);
        Ok(Some(data))
    }

    fn encode_data(
        &mut self,
        packet_id: u16,
        body: Bytes,
    ) -> io::Result<Bytes> {
        let n = body.len() + 4;
        let mut result = BytesMut::with_capacity(n);
        result.put_u16_le(n as u16); // packet length (0) -> (2)
        result.put_u16_le(packet_id); // packet type (2) -> (4)
        result.extend_from_slice(&body); // packet_body (4) -> (packet_length)
        let full_packet = result.freeze();
        debug!(
            "\nServer -> Client ID({}) {:?}",
            packet_id,
            full_packet.as_ref().hex_dump()
        );
        let mut encrypted_data = BytesMut::with_capacity(n);
        encrypted_data.resize(n, 0);
        // encrypt data
        self.cipher.encrypt(&full_packet, &mut encrypted_data);
        Ok(encrypted_data.freeze())
    }
}

impl<S> Stream for TQCodec<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Item = io::Result<(u16, Bytes)>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // First, read any new data that might have been received off the socket
        let sock_closed = {
            let r = self.fill_read_buf();
            tokio::pin!(r);
            match r.poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(res) => res.is_err(),
            }
        };
        let (n, packet_id) = match self.state {
            DecodeState::Head => match self.decode_head()? {
                Some((n, packet_id)) => {
                    self.state = DecodeState::Data((n, packet_id));
                    (n, packet_id)
                },
                None => {
                    if sock_closed {
                        // we know we will get here when we read zero from the
                        // socket so we need to stop
                        // looping and just free all resources
                        return Poll::Ready(None);
                    } else {
                        return Poll::Pending;
                    }
                },
            },
            DecodeState::Data((n, packet_id)) => (n, packet_id),
        };
        if let Some(data) = self.decode_data(n)? {
            debug!(
                "\nClient -> Server ID({}) {:?}",
                packet_id,
                data.as_ref().hex_dump()
            );
            // Update the decode state
            self.state = DecodeState::Head;
            let data = Ok((packet_id, data.freeze()));
            return Poll::Ready(Some(data));
        }

        if sock_closed {
            warn!("Socket Closed, end of stream!");
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}
