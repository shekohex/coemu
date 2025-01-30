//! crate that contians a [`TQCodec`] that wraps any [`AsyncRead`] +
//! [`AsyncWrite`] and Outputs a Stream-like [`TQDecoder`] of `(u16, Bytes)`
//! where the `u16`s the are the PacketID and Bytes is the Body of the Packet.
//! It also implements Sink-like
//! [`TQEncoder`] where you could write `(u16, Bytes)` to it.
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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use pretty_hex::{HexConfig, PrettyHex};
use tokio::io::{self, split, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio_stream::Stream;
use tq_crypto::Cipher;

const KB: usize = 1024;
const MB: usize = 1024 * KB;
const MAX_PACKET_SIZE: u16 = 2 * KB as u16;
const MAX_CAPACITY: usize = 10 * MB;

/// A simple State Machine for Decoding the Stream.
#[derive(Debug, Clone, Copy)]
enum DecodeState {
    /// Decoding the Head of the Packet (4 Bytes for the Size + PacketID)
    Head,
    /// Decoding the Packet Bytes (usize => Packet Size, u16 => PacketID)
    Data((usize, u16)),
}

#[derive(Debug)]
pub struct TQDecoder<S: AsyncRead + AsyncWrite, C: Cipher> {
    /// Current Decode State
    state: DecodeState,
    /// Cipher Used to Decrypt Packets
    cipher: C,
    /// Buffer used when reading from the stream. Data is not returned from
    /// this buffer until an entire packet has been read.
    buf: BytesMut,
    /// The Underlaying Read Half of Socket
    rdr: ReadHalf<S>,
}

impl<S: AsyncRead + AsyncWrite, C: Cipher> TQDecoder<S, C> {
    /// Read data from the socket.
    ///
    /// This only returns `Ready` when the socket has closed.
    fn fill_read_buf(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        loop {
            // Ensure the read buffer has capacity.
            //
            // This might result in an internal allocation.
            self.buf.reserve(64);

            // Read data into the buffer.
            let n: usize = {
                let p = self.rdr.read_buf(&mut self.buf);
                tokio::pin!(p);
                match p.poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(v) => v?,
                }
            };

            if n == 0 {
                return Poll::Ready(Ok(()));
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn decode_head(&mut self) -> io::Result<Option<(usize, u16)>> {
        tracing::trace!(buf_len = %self.buf.len(), "buffer bytes");
        if self.buf.len() < 4 {
            // Not enough data
            tracing::trace!("no enough data");
            return Ok(None);
        }
        let (n, packet_type) = {
            let mut len = self.buf.split_to(2);
            let mut ty = self.buf.split_to(2);
            // Get the decrypted head len.
            self.cipher.decrypt(&mut len);
            // Get the decrypted head packet type.
            self.cipher.decrypt(&mut ty);
            // get length
            let n = len.as_ref().get_u16_le();
            // get type
            let packet_id = ty.as_ref().get_u16_le();
            tracing::trace!(%n, %packet_id, "decoded head");
            if n > MAX_PACKET_SIZE {
                tracing::warn!(%n, %packet_id, "Frame too big!");
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Frame Too Big"));
            }
            (n as usize, packet_id)
        };
        // Ensure that the buffer has enough space to read the incoming
        // payload
        self.buf.reserve(n - 4);

        Ok(Some((n - 4, packet_type)))
    }

    #[tracing::instrument(skip(self))]
    fn decode_data(&mut self, n: usize) -> io::Result<Option<BytesMut>> {
        tracing::trace!(
            data_len = %n,
            current_buf_len = %self.buf.len(),
            "decoding data"
        );

        // At this point, the buffer has already had the required capacity
        // reserved. All there is to do is read.

        if self.buf.len() < n {
            tracing::trace!("Buffer too small, skipping");
            return Ok(None);
        }
        let mut buf = self.buf.split_to(n);
        self.cipher.decrypt(&mut buf);
        Ok(Some(buf))
    }
}

pub struct TQEncoder<S: AsyncRead + AsyncWrite, C: Cipher> {
    /// Cipher Used to Encrypt Packets
    cipher: C,
    /// Buffer used to stage data before writing it to the socket.
    buf: BytesMut,
    /// The Underlaying Write Half of Socket
    wrt: WriteHalf<S>,
}

impl<S: AsyncRead + AsyncWrite, C: Cipher> TQEncoder<S, C> {
    /// Send Item to the Underlaying Socket.
    ///
    /// Items (u16, Bytes) got Encoded and Encrypted and sent to socket.
    #[tracing::instrument(skip(self, item))]
    pub async fn send(&mut self, item: (u16, Bytes)) -> Result<(), io::Error> {
        let buf = self.encode_data(item.0, item.1)?;
        self.buffer(&buf);
        self.flush().await?;
        Ok(())
    }

    /// Close The Socket .. No More IO.
    #[tracing::instrument(skip(self))]
    pub async fn close(&mut self) -> Result<(), io::Error> {
        tracing::trace!("Sutting down socket");
        self.wrt.shutdown().await?;
        Ok(())
    }

    #[tracing::instrument(skip(self, body))]
    fn encode_data(&self, packet_id: u16, body: Bytes) -> io::Result<Bytes> {
        tracing::trace!(%packet_id, "encoding packet");
        let n = body.len() + 4;
        let mut result = BytesMut::with_capacity(n);
        result.put_u16_le(n as u16); // packet length (0) -> (2)
        result.put_u16_le(packet_id); // packet type (2) -> (4)
        result.extend_from_slice(&body); // packet_body (4) -> (packet_length)
        let mut full_packet = result;
        let config = HexConfig {
            title: false,
            ..Default::default()
        };
        tracing::trace!(
            "\nServer -> Client ID({packet_id}) Length({n})\n{:?}",
            body.as_ref().hex_conf(config)
        );
        // encrypt data
        self.cipher.encrypt(&mut full_packet);
        Ok(full_packet.freeze())
    }

    /// Buffer a packet.
    ///
    /// This writes the packet to an internal buffer. Calls to `flush` will
    /// attempt to flush this buffer to the socket.
    #[tracing::instrument(skip(self, buf))]
    fn buffer(&mut self, buf: &[u8]) {
        // Ensure the buffer has capacity. Ideally this would not be unbounded,
        // but to keep the example simple, we will not limit this.
        if self.buf.capacity() <= buf.len() && self.buf.capacity() < MAX_CAPACITY {
            self.buf.reserve(buf.len());
        }

        // Push the packet onto the end of the write buffer.
        //
        // The `put` function is from the `BufMut` trait.
        self.buf.put(buf);
        tracing::trace!(
            buf_len = %buf.len(),
            "bytes got buffered and ready to be sent",
        );
    }

    /// Flush the write buffer to the socket
    #[tracing::instrument(skip(self))]
    async fn flush(&mut self) -> Result<(), io::Error> {
        tracing::trace!("flushing data into stream");
        // As long as there is buffered data to write, try to write it.
        while self.buf.has_remaining() {
            let n = self.wrt.write_buf(&mut self.buf).await?;
            tracing::trace!("written {} bytes", n);
        }
        self.wrt.flush().await?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TQCodec<S: AsyncRead + AsyncWrite, C: Cipher + Clone> {
    stream: S,
    cipher: C,
}

impl<S: AsyncRead + AsyncWrite, C: Cipher + Clone> TQCodec<S, C> {
    pub fn new(stream: S, cipher: C) -> Self {
        Self { stream, cipher }
    }

    pub fn split(self) -> (TQEncoder<S, C>, TQDecoder<S, C>) {
        let (rdr, wrt) = split(self.stream);
        let encoder = TQEncoder {
            buf: BytesMut::with_capacity(64),
            cipher: self.cipher.clone(),
            wrt,
        };
        let decoder = TQDecoder {
            state: DecodeState::Head,
            buf: BytesMut::with_capacity(64),
            cipher: self.cipher,
            rdr,
        };
        (encoder, decoder)
    }
}

impl<S, C> Stream for TQDecoder<S, C>
where
    S: AsyncRead + AsyncWrite + Unpin,
    C: Cipher + Unpin,
{
    type Item = io::Result<(u16, Bytes)>;

    #[tracing::instrument(skip(self, cx))]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // First, read any new data that might have been received off the socket
        let sock_closed = self.fill_read_buf(cx)?.is_ready();
        tracing::trace!("Socket Close? {}", sock_closed);
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
            let config = HexConfig {
                title: false,
                ..Default::default()
            };
            let packet_len = n + 4;
            tracing::trace!(
                "\nClient -> Server ID({packet_id}) Length({packet_len})\n{:?}",
                data.as_ref().hex_conf(config)
            );
            // Update the decode state
            self.state = DecodeState::Head;
            let data = Ok((packet_id, data.freeze()));
            return Poll::Ready(Some(data));
        }

        if sock_closed {
            tracing::warn!("Socket Closed, end of stream!");
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}
