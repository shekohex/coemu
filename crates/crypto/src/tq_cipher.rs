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

use core::sync::atomic::{AtomicU16, AtomicU8, Ordering};
use parking_lot::RwLock;

#[cfg(not(feature = "std"))]
use alloc::sync::Arc;
#[cfg(feature = "std")]
use std::sync::Arc;

const KEY_SIZE: usize = 0x200;
const C: usize = KEY_SIZE / 2;

const SEED: [u8; 8] = [
    0x9D, 0x0F, 0xFA, 0x13, // P: 0x13FA0F9D
    0x62, 0x79, 0x5C, 0x6D, // G: 0x6D5C7962
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
    key1: Arc<RwLock<[u8; KEY_SIZE]>>,
    key2: Arc<RwLock<[u8; KEY_SIZE]>>,
    active_key: Arc<AtomicU8>,
    decrypt_counter: Arc<AtomicU16>,
    encrypt_counter: Arc<AtomicU16>,
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum ActiveKey {
    Key1 = 0,
    Key2 = 1,
}

impl From<u8> for ActiveKey {
    fn from(v: u8) -> Self {
        match v {
            0 => ActiveKey::Key1,
            1 => ActiveKey::Key2,
            _ => unreachable!("Invalid ActiveKey"),
        }
    }
}

impl TQCipher {
    /// Instantiates a new instance of TQCipher using pregenerated
    /// IVs for initializing the cipher's keystreams. Initialized on each server
    /// to start communication. The game server will also require that keys are
    /// regenerated using key derivations from the client's first packet.
    /// Increments counters with thread-safety for synchronous reads and
    /// writes.
    pub fn new() -> Self {
        let mut seed = SEED;
        let mut k = [0u8; KEY_SIZE];
        for i in 0..C {
            k[i] = seed[0];
            k[i + C] = seed[4];
            // seed[0] = (byte)((seed[1] + (seed[0] * seed[2])) * seed[0] +
            // seed[3]);
            {
                let a = seed[0].wrapping_mul(seed[2]);
                let b = seed[1].wrapping_add(a);
                let c = seed[0].wrapping_mul(b);
                let d = seed[3].wrapping_add(c);
                seed[0] = d;
            }
            // seed[4] = (byte)((seed[5] - (seed[4] * seed[6])) * seed[4] +
            // seed[7]);
            {
                let a = seed[4].wrapping_mul(seed[6]);
                let b = seed[5].wrapping_sub(a);
                let c = seed[4].wrapping_mul(b);
                let d = seed[7].wrapping_add(c);
                seed[4] = d;
            }
        }
        Self {
            key1: Arc::new(RwLock::new(k)),
            key2: Arc::new(RwLock::new(k)),
            active_key: Arc::new(AtomicU8::new(ActiveKey::Key1 as u8)),
            decrypt_counter: Arc::new(AtomicU16::new(0)),
            encrypt_counter: Arc::new(AtomicU16::new(0)),
        }
    }

    #[inline(always)]
    fn xor_in_place(src: &mut [u8], key: &[u8; KEY_SIZE], counter: &AtomicU16) {
        let mut x = counter.fetch_add(src.len() as u16, Ordering::SeqCst);
        (0..src.len()).for_each(|i| {
            src[i] ^= 0xAB;
            src[i] = src[i] >> 4 | src[i] << 4;
            src[i] ^= key[(x & 0xff) as usize];
            src[i] ^= key[((x >> 8) + 0x100) as usize];
            x = x.wrapping_add(1);
        });
    }
}

impl super::Cipher for TQCipher {
    /// Generates keys for the game server using the player's server access
    /// token as a key derivation variable. Invoked after the first packet
    /// is received on the game server.
    fn generate_keys(&self, seed: u64) {
        let a = (seed >> 32) as u32;
        let b = seed as u32;
        let c = (a.wrapping_add(b) ^ 0x4321) ^ a;
        let d = c.wrapping_mul(c);

        let tmp1 = c.to_le_bytes();
        let tmp2 = d.to_le_bytes();
        let mut key2 = self.key2.write();
        let key1 = self.key1.read();
        for i in 0..C {
            key2[i] = key1[i] ^ tmp1[i % 4];
            key2[i + C] = key1[i + C] ^ tmp2[i % 4];
        }
        self.active_key.store(ActiveKey::Key2 as u8, Ordering::SeqCst);
        self.encrypt_counter.store(0, Ordering::SeqCst);
    }

    /// Decrypts the specified slice by XORing the source slice with the
    /// cipher's keystream.
    fn decrypt(&self, data: &mut [u8]) {
        let active_key = self.active_key.load(Ordering::SeqCst);
        let key = match ActiveKey::from(active_key) {
            ActiveKey::Key1 => self.key1.read(),
            ActiveKey::Key2 => self.key2.read(),
        };
        Self::xor_in_place(data, &key, &self.decrypt_counter);
    }

    /// Encrypt the specified slice by XORing the source slice with the cipher's
    /// keystream.
    fn encrypt(&self, data: &mut [u8]) {
        let key = self.key1.read();
        Self::xor_in_place(data, &key, &self.encrypt_counter);
    }
}

impl Default for TQCipher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::Cipher;

    use super::*;

    #[test]
    fn tq_cipher() {
        let tq_cipher = TQCipher::new();
        tq_cipher.generate_keys(0x1234);
        let mut buffer = [
            0x22, 0x00, 0x1F, 0x04, 0x61, 0xFF, 0xC3, 0xA6, 0x3A, 0x6D, 0xD3, 0x90, 0x31, 0x39, 0x32, 0x2E, 0x31, 0x36,
            0x38, 0x2E, 0x31, 0x2E, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00, 0xB8, 0x16, 0x00, 0x00, 0x00, 0x00,
        ];
        tq_cipher.encrypt(&mut buffer);
        assert_eq!(
            buffer,
            [
                0x67, 0x48, 0xAA, 0x12, 0x1F, 0xAB, 0x3, 0x44, 0x5E, 0x26, 0xE, 0x53, 0x52, 0x2F, 0x74, 0x14, 0xE6,
                0xFB, 0x88, 0xC0, 0x2A, 0x86, 0x4C, 0x3E, 0x6D, 0x0, 0xE3, 0x2A, 0xFA, 0x2D, 0x87, 0xC6, 0x65, 0x28
            ]
        );
    }
}
