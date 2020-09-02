//! crate that contians a [`TQCodec`] that wraps any [`AsyncRead`] +
//! [`AsyncWrite`] and Outputs a Stream-like [`TQDecoder`] of `(u16, Bytes)`
//! where the `u16` is the PacketID and Bytes is the Body of the Packet.
//! It also implements Sink-like [`TQEncoder`] where you could write `(u16,
//! Bytes)` to it.
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
use pretty_hex::PrettyHex;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{
        split, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf,
        WriteHalf,
    },
    stream::Stream,
};
use tq_crypto::Cipher;
use tracing::{instrument, trace, warn};

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
    fn fill_read_buf(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
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

    #[instrument(skip(self))]
    fn decode_head(&mut self) -> io::Result<Option<(usize, u16)>> {
        trace!("buf len {} bytes", self.buf.len());
        if self.buf.len() < 4 {
            // Not enough data
            trace!("no enough data");
            return Ok(None);
        }
        let (n, packet_type) = {
            let mut len = [0u8; 2];
            let mut ty = [0u8; 2];
            // Get the decrypted head len.
            self.cipher.decrypt(&self.buf[0..2], &mut len);
            // Get the decrypted head packet type.
            self.cipher.decrypt(&self.buf[2..4], &mut ty);
            // get length
            let n = len.as_ref().get_u16_le();
            // get type
            let packet_id = ty.as_ref().get_u16_le();
            trace!("len {}, id {}", n, packet_id);
            if n > 5_000 {
                trace!("Frame too big!");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Frame Too Big",
                ));
            }
            (n as usize, packet_id)
        };
        // Ensure that the buffer has enough space to read the incoming
        // payload
        self.buf.reserve(n - 4);
        // Drop the header
        let _ = self.buf.split_to(4);

        Ok(Some((n - 4, packet_type)))
    }

    #[instrument(skip(self))]
    fn decode_data(&mut self, n: usize) -> io::Result<Option<BytesMut>> {
        trace!("buf len {}, len {}", self.buf.len(), n);
        // At this point, the buffer has already had the required capacity
        // reserved. All there is to do is read.

        if self.buf.len() < n {
            trace!("Buffer too small");
            return Ok(None);
        }
        let buf = &mut self.buf.split_to(n);
        let mut data = BytesMut::with_capacity(n);
        data.resize(n, 0);
        // Decrypt the data
        trace!("Decrypt Buffer");
        self.cipher.decrypt(&buf, &mut data);
        Ok(Some(data))
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
    #[instrument(skip(self, item))]
    pub async fn send(&mut self, item: (u16, Bytes)) -> Result<(), io::Error> {
        trace!("Encoding Packet ({})", item.0);
        let buf = self.encode_data(item.0, item.1)?;
        trace!("Packet ({}) Encoded", item.0);
        self.buffer(&buf);
        trace!("Packet ({}) Buffered", item.0);
        self.flush().await?;
        trace!("Packet ({}) Flushed", item.0);
        Ok(())
    }

    /// Close The Socket .. No More IO.
    #[instrument(skip(self))]
    pub async fn close(&mut self) -> Result<(), io::Error> {
        trace!("Sutting down socket");
        self.wrt.shutdown().await?;
        Ok(())
    }

    #[instrument(skip(self, body))]
    fn encode_data(&self, packet_id: u16, body: Bytes) -> io::Result<Bytes> {
        let n = body.len() + 4;
        let mut result = BytesMut::with_capacity(n);
        result.put_u16_le(n as u16); // packet length (0) -> (2)
        result.put_u16_le(packet_id); // packet type (2) -> (4)
        result.extend_from_slice(&body); // packet_body (4) -> (packet_length)
        let full_packet = result.freeze();
        trace!(
            "\nServer -> Client ID({}) {:?}",
            packet_id,
            full_packet.as_ref().hex_dump()
        );
        let mut encrypted_data = BytesMut::with_capacity(n);
        encrypted_data.resize(n, 0);
        // encrypt data
        trace!("Encrypt data");
        self.cipher.encrypt(&full_packet, &mut encrypted_data);
        Ok(encrypted_data.freeze())
    }

    /// Buffer a packet.
    ///
    /// This writes the packet to an internal buffer. Calls to `flush` will
    /// attempt to flush this buffer to the socket.
    #[instrument(skip(self, buf))]
    fn buffer(&mut self, buf: &[u8]) {
        // Ensure the buffer has capacity. Ideally this would not be unbounded,
        // but to keep the example simple, we will not limit this.
        self.buf.reserve(64);

        // Push the packet onto the end of the write buffer.
        //
        // The `put` function is from the `BufMut` trait.
        self.buf.put(buf);
        trace!("{} bytes got buffered and ready to be sent", buf.len());
    }

    /// Flush the write buffer to the socket
    #[instrument(skip(self))]
    async fn flush(&mut self) -> Result<(), io::Error> {
        trace!("flushing data into stream");
        // As long as there is buffered data to write, try to write it.
        let n = self.buf.len();
        while self.buf.has_remaining() {
            let n = self.wrt.write_buf(&mut self.buf).await?;
            trace!("written {} bytes", n);
        }
        self.wrt.flush().await?;
        trace!("flushed {} bytes", n);
        Ok(())
    }
}

#[derive(Debug)]
pub struct TQCodec<S: AsyncRead + AsyncWrite, C: Cipher + Clone> {
    stream: S,
    cipher: C,
}

impl<S: AsyncRead + AsyncWrite, C: Cipher + Clone> TQCodec<S, C> {
    pub fn new(stream: S, cipher: C) -> Self { Self { stream, cipher } }

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

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // First, read any new data that might have been received off the socket
        let sock_closed = self.fill_read_buf(cx)?.is_ready();
        trace!("Socket Close? {}", sock_closed);
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
            trace!(
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
