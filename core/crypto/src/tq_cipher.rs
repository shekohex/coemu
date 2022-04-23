//! This cipher algorithm is implemented in the Account Server for all patches
//! above 4232, and in the game server for patches 4232 - 5017. The cipher was
//! replaced in the game server in patch 5018 by Blowfish. Keys in this
//! implementation are targeted at the Conquer Online game client. This
//! implementation was programmed by [CptSky][1] in his [COPS v6][2] project.
//!
//! For More info see ConquerWiki page about [TQ Digital Server Asymmetric
//! Cipher][3].
//!
//! [1]: https://www.elitepvpers.com/forum/members/568265-cptsky.html
//! [2]: https://www.elitepvpers.com/forum/co2-pserver-guides-releases/2402439-cops-v6-source-tools-custom-emulator.html
//! [3]: https://www.forum.darkfoxdeveloper.com/conquerwiki/doku.php?id=conqueronlineserverasymmetriccipher
use crate::Cipher;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use once_cell::sync::OnceCell;
use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        Arc, Mutex,
    },
};
const K: usize = 256;

const KEY1: [u8; K] = [
    0x9D, 0x90, 0x83, 0x8A, 0xD1, 0x8C, 0xE7, 0xF6, 0x25, 0x28, 0xEB, 0x82,
    0x99, 0x64, 0x8F, 0x2E, 0x2D, 0x40, 0xD3, 0xFA, 0xE1, 0xBC, 0xB7, 0xE6,
    0xB5, 0xD8, 0x3B, 0xF2, 0xA9, 0x94, 0x5F, 0x1E, 0xBD, 0xF0, 0x23, 0x6A,
    0xF1, 0xEC, 0x87, 0xD6, 0x45, 0x88, 0x8B, 0x62, 0xB9, 0xC4, 0x2F, 0x0E,
    0x4D, 0xA0, 0x73, 0xDA, 0x01, 0x1C, 0x57, 0xC6, 0xD5, 0x38, 0xDB, 0xD2,
    0xC9, 0xF4, 0xFF, 0xFE, 0xDD, 0x50, 0xC3, 0x4A, 0x11, 0x4C, 0x27, 0xB6,
    0x65, 0xE8, 0x2B, 0x42, 0xD9, 0x24, 0xCF, 0xEE, 0x6D, 0x00, 0x13, 0xBA,
    0x21, 0x7C, 0xF7, 0xA6, 0xF5, 0x98, 0x7B, 0xB2, 0xE9, 0x54, 0x9F, 0xDE,
    0xFD, 0xB0, 0x63, 0x2A, 0x31, 0xAC, 0xC7, 0x96, 0x85, 0x48, 0xCB, 0x22,
    0xF9, 0x84, 0x6F, 0xCE, 0x8D, 0x60, 0xB3, 0x9A, 0x41, 0xDC, 0x97, 0x86,
    0x15, 0xF8, 0x1B, 0x92, 0x09, 0xB4, 0x3F, 0xBE, 0x1D, 0x10, 0x03, 0x0A,
    0x51, 0x0C, 0x67, 0x76, 0xA5, 0xA8, 0x6B, 0x02, 0x19, 0xE4, 0x0F, 0xAE,
    0xAD, 0xC0, 0x53, 0x7A, 0x61, 0x3C, 0x37, 0x66, 0x35, 0x58, 0xBB, 0x72,
    0x29, 0x14, 0xDF, 0x9E, 0x3D, 0x70, 0xA3, 0xEA, 0x71, 0x6C, 0x07, 0x56,
    0xC5, 0x08, 0x0B, 0xE2, 0x39, 0x44, 0xAF, 0x8E, 0xCD, 0x20, 0xF3, 0x5A,
    0x81, 0x9C, 0xD7, 0x46, 0x55, 0xB8, 0x5B, 0x52, 0x49, 0x74, 0x7F, 0x7E,
    0x5D, 0xD0, 0x43, 0xCA, 0x91, 0xCC, 0xA7, 0x36, 0xE5, 0x68, 0xAB, 0xC2,
    0x59, 0xA4, 0x4F, 0x6E, 0xED, 0x80, 0x93, 0x3A, 0xA1, 0xFC, 0x77, 0x26,
    0x75, 0x18, 0xFB, 0x32, 0x69, 0xD4, 0x1F, 0x5E, 0x7D, 0x30, 0xE3, 0xAA,
    0xB1, 0x2C, 0x47, 0x16, 0x05, 0xC8, 0x4B, 0xA2, 0x79, 0x04, 0xEF, 0x4E,
    0x0D, 0xE0, 0x33, 0x1A, 0xC1, 0x5C, 0x17, 0x06, 0x95, 0x78, 0x9B, 0x12,
    0x89, 0x34, 0xBF, 0x3E,
];

const KEY2: [u8; K] = [
    0x62, 0x4F, 0xE8, 0x15, 0xDE, 0xEB, 0x04, 0x91, 0x1A, 0xC7, 0xE0, 0x4D,
    0x16, 0xE3, 0x7C, 0x49, 0xD2, 0x3F, 0xD8, 0x85, 0x4E, 0xDB, 0xF4, 0x01,
    0x8A, 0xB7, 0xD0, 0xBD, 0x86, 0xD3, 0x6C, 0xB9, 0x42, 0x2F, 0xC8, 0xF5,
    0xBE, 0xCB, 0xE4, 0x71, 0xFA, 0xA7, 0xC0, 0x2D, 0xF6, 0xC3, 0x5C, 0x29,
    0xB2, 0x1F, 0xB8, 0x65, 0x2E, 0xBB, 0xD4, 0xE1, 0x6A, 0x97, 0xB0, 0x9D,
    0x66, 0xB3, 0x4C, 0x99, 0x22, 0x0F, 0xA8, 0xD5, 0x9E, 0xAB, 0xC4, 0x51,
    0xDA, 0x87, 0xA0, 0x0D, 0xD6, 0xA3, 0x3C, 0x09, 0x92, 0xFF, 0x98, 0x45,
    0x0E, 0x9B, 0xB4, 0xC1, 0x4A, 0x77, 0x90, 0x7D, 0x46, 0x93, 0x2C, 0x79,
    0x02, 0xEF, 0x88, 0xB5, 0x7E, 0x8B, 0xA4, 0x31, 0xBA, 0x67, 0x80, 0xED,
    0xB6, 0x83, 0x1C, 0xE9, 0x72, 0xDF, 0x78, 0x25, 0xEE, 0x7B, 0x94, 0xA1,
    0x2A, 0x57, 0x70, 0x5D, 0x26, 0x73, 0x0C, 0x59, 0xE2, 0xCF, 0x68, 0x95,
    0x5E, 0x6B, 0x84, 0x11, 0x9A, 0x47, 0x60, 0xCD, 0x96, 0x63, 0xFC, 0xC9,
    0x52, 0xBF, 0x58, 0x05, 0xCE, 0x5B, 0x74, 0x81, 0x0A, 0x37, 0x50, 0x3D,
    0x06, 0x53, 0xEC, 0x39, 0xC2, 0xAF, 0x48, 0x75, 0x3E, 0x4B, 0x64, 0xF1,
    0x7A, 0x27, 0x40, 0xAD, 0x76, 0x43, 0xDC, 0xA9, 0x32, 0x9F, 0x38, 0xE5,
    0xAE, 0x3B, 0x54, 0x61, 0xEA, 0x17, 0x30, 0x1D, 0xE6, 0x33, 0xCC, 0x19,
    0xA2, 0x8F, 0x28, 0x55, 0x1E, 0x2B, 0x44, 0xD1, 0x5A, 0x07, 0x20, 0x8D,
    0x56, 0x23, 0xBC, 0x89, 0x12, 0x7F, 0x18, 0xC5, 0x8E, 0x1B, 0x34, 0x41,
    0xCA, 0xF7, 0x10, 0xFD, 0xC6, 0x13, 0xAC, 0xF9, 0x82, 0x6F, 0x08, 0x35,
    0xFE, 0x0B, 0x24, 0xB1, 0x3A, 0xE7, 0x00, 0x6D, 0x36, 0x03, 0x9C, 0x69,
    0xF2, 0x5F, 0xF8, 0xA5, 0x6E, 0xFB, 0x14, 0x21, 0xAA, 0xD7, 0xF0, 0xDD,
    0xA6, 0xF3, 0x8C, 0xD9,
];

/// TQ Digital Entertainment's in-house asymmetric counter-based XOR-cipher.
/// Counters are separated by encryption direction to create cipher streams.
/// This implementation implements both directions for encrypting and decrypting
/// data on the server side.
///
/// This cipher algorithm does not provide effective security, and does not make
/// use of any NP-hard calculations for encryption or key generation. Key
/// derivations are susceptible to brute-force or static key attacks. Only
/// implemented for interoperability with the pre-existing game client. Do not
/// use, otherwise.
#[derive(Clone)]
pub struct TQCipher {
    key3: Arc<OnceCell<Bytes>>,
    key4: Arc<OnceCell<Bytes>>,
    use_alt_key: Arc<AtomicBool>,
    decrypt_counter: Arc<AtomicI64>,
    encrypt_counter: Arc<AtomicI64>,
}

impl TQCipher {
    /// Instantiates a new instance of TQCipher using pregenerated
    /// IVs for initializing the cipher's keystreams. Initialized on each server
    /// to start communication. The game server will also require that keys are
    /// regenerated using key derivations from the client's first packet.
    /// Increments counters without thread-safety for synchronous reads and
    /// writes.
    pub fn new() -> Self {
        TQCipher {
            key3: Arc::new(OnceCell::new()),
            key4: Arc::new(OnceCell::new()),
            use_alt_key: Arc::new(AtomicBool::new(false)),
            decrypt_counter: Arc::new(AtomicI64::new(0)),
            encrypt_counter: Arc::new(AtomicI64::new(0)),
        }
    }
}

impl fmt::Debug for TQCipher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TQCipher {{ decrypt_counter: {:?}, encrypt_counter: {:?}, use_alt_key: {:?} }}",
            self.decrypt_counter, self.encrypt_counter, self.use_alt_key,
        )
    }
}

impl Default for TQCipher {
    fn default() -> Self {
        Self::new()
    }
}

impl Cipher for TQCipher {
    /// Generates keys for the game server using the player's server access
    /// token as a key derivation variable. Invoked after the first packet
    /// is received on the game server.
    #[inline(always)]
    fn generate_keys(&self, a: u32, b: u32) {
        let tmp1 = (a.wrapping_add(b) ^ 0x4321) ^ a;
        let tmp2 = tmp1.wrapping_mul(tmp1);
        const C: usize = K / 4;
        let mut key1 = Bytes::from_static(&KEY1);
        let mut key2 = Bytes::from_static(&KEY2);
        let mut key3 = BytesMut::with_capacity(C);
        let mut key4 = BytesMut::with_capacity(C);
        for _ in 0..C {
            let k1 = key1.get_u32_le();
            let k2 = key2.get_u32_le();
            key3.put_u32_le(tmp1 ^ k1);
            key4.put_u32_le(tmp2 ^ k2);
        }
        self.key3
            .set(key3.freeze())
            .expect("Keys already generated once!");
        self.key4
            .set(key4.freeze())
            .expect("Keys already generated once!");
        self.encrypt_counter.store(0, Ordering::Relaxed);
        self.use_alt_key.store(true, Ordering::Relaxed);
    }

    /// Decrypts the specified slice by XORing the source slice with the
    /// cipher's keystream. The source and destination may be the same
    /// slice, but otherwise should not overlap.
    #[inline(always)]
    fn decrypt(&self, src: &[u8], dst: &mut [u8]) {
        assert_eq!(src.len(), dst.len());
        let key3 = self.key3.get().expect("Keys not generated!");
        let key4 = self.key4.get().expect("Keys not generated!");
        for i in 0..src.len() {
            dst[i] = src[i] ^ 0xAB;
            dst[i] = dst[i] << 4 | dst[i] >> 4;
            let decrypt_counter = self.decrypt_counter.load(Ordering::Relaxed);
            if self.use_alt_key.load(Ordering::Relaxed) {
                dst[i] ^= key4[(decrypt_counter >> 8) as usize];
                dst[i] ^= key3[(decrypt_counter & 0xFF) as usize];
            } else {
                dst[i] ^= KEY2[(decrypt_counter >> 8) as usize];
                dst[i] ^= KEY1[(decrypt_counter & 0xFF) as usize];
            }
            self.decrypt_counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Encrypt the specified slice by XORing the source slice with the cipher's
    /// keystream. The source and destination may be the same slice, but
    /// otherwise should not overlap.
    #[inline(always)]
    fn encrypt(&self, src: &[u8], dst: &mut [u8]) {
        assert_eq!(src.len(), dst.len());
        for i in 0..src.len() {
            let encrypt_counter = self.encrypt_counter.load(Ordering::Relaxed);
            dst[i] = src[i] ^ 0xAB;
            dst[i] = dst[i] << 4 | dst[i] >> 4;
            dst[i] ^= KEY2[(encrypt_counter >> 8) as usize];
            dst[i] ^= KEY1[(encrypt_counter & 0xFF) as usize];
            self.encrypt_counter.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TQCipher;
    use crate::Cipher;
    #[test]
    fn test_tq_cipher() {
        let tq_cipher = TQCipher::new();
        let buffer = [
            0x22, 0x00, 0x1F, 0x04, 0x61, 0xFF, 0xC3, 0xA6, 0x3A, 0x6D, 0xD3,
            0x90, 0x31, 0x39, 0x32, 0x2E, 0x31, 0x36, 0x38, 0x2E, 0x31, 0x2E,
            0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0xB8, 0x16, 0x00, 0x00, 0x00,
            0x00,
        ];
        let mut encrypted = vec![0u8; buffer.len()];
        tq_cipher.encrypt(&buffer, &mut encrypted);
        assert_eq!(
            encrypted,
            vec![
                0x67, 0x48, 0xAA, 0x12, 0x1F, 0xAB, 0x3, 0x44, 0x5E, 0x26, 0xE,
                0x53, 0x52, 0x2F, 0x74, 0x14, 0xE6, 0xFB, 0x88, 0xC0, 0x2A,
                0x86, 0x4C, 0x3E, 0x6D, 0x0, 0xE3, 0x2A, 0xFA, 0x2D, 0x87,
                0xC6, 0x65, 0x28
            ]
        );
    }
}
