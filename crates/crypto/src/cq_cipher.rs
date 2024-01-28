//! This cipher algorithm is implemented in the client for all patches above
//! 4232. Keys in this implementation are targeted at the Conquer Online game
//! client. This implementation was programmed by [CptSky][1] in his
//! [Co2_Core_Dll project][2] project.
//!
//! For More info see ConquerWiki page about [ TQ Digital Client Asymmetric
//! Cipher][3].
//!
//! [1]: https://www.elitepvpers.com/forum/members/568265-cptsky.html
//! [2]: https://www.elitepvpers.com/forum/co2-pserver-guides-releases/1652536-co2_core_dll-c-library.html
//! [3]: https://www.conquerwiki.com/doku.php?id=conqueronlineclientasymmetriccipher
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

/// Conquer Online Client Asymmetric Cipher.
#[derive(Clone)]
pub struct CQCipher {
    key1: Arc<RwLock<[u8; KEY_SIZE]>>,
    key2: Arc<RwLock<[u8; KEY_SIZE]>>,
    active_key: Arc<AtomicU8>,
    decrypt_counter: Arc<AtomicU16>,
    encrypt_counter: Arc<AtomicU16>,
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum ActiveKey {
    /// IV is generated from key1.
    Key1 = 0,
    /// Client's Generated Key.
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

impl CQCipher {
    /// Generates an initialization vector (IV) to use for the algorithm.
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
}

impl super::Cipher for CQCipher {
    /// Generates a key (Key) to use for the algorithm and reset the decrypt
    /// counter.
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
        self.decrypt_counter.store(0, Ordering::SeqCst);
    }

    /// Decrypts data with the COCAC algorithm.
    fn decrypt(&self, data: &mut [u8]) {
        let key1 = self.key1.read();
        let mut x = self.decrypt_counter.fetch_add(data.len() as u16, Ordering::SeqCst);
        (0..data.len()).for_each(|i| {
            data[i] ^= key1[((x >> 8) + 0x100) as usize];
            data[i] ^= key1[(x & 0xff) as usize];
            data[i] = data[i] >> 4 | data[i] << 4;
            data[i] ^= 0xAB;
            x = x.wrapping_add(1);
        });
    }

    /// Encrypts data with the COCAC algorithm..
    fn encrypt(&self, data: &mut [u8]) {
        let active_key_value = self.active_key.load(Ordering::SeqCst);
        let active_key = ActiveKey::from(active_key_value);
        let mut x = self.encrypt_counter.fetch_add(data.len() as u16, Ordering::SeqCst);
        let key1 = self.key1.read();
        let key2 = self.key2.read();
        (0..data.len()).for_each(|i| {
            match active_key {
                ActiveKey::Key1 => {
                    data[i] ^= key1[((x >> 8) + 0x100) as usize];
                    data[i] ^= key1[(x & 0xff) as usize];
                },
                ActiveKey::Key2 => {
                    data[i] ^= key2[((x >> 8) + 0x100) as usize];
                    data[i] ^= key2[(x & 0xff) as usize];
                },
            }
            data[i] = data[i] >> 4 | data[i] << 4;
            data[i] ^= 0xAB;
            x = x.wrapping_add(1);
        });
    }
}

impl Default for CQCipher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    use crate::{Cipher, TQCipher};

    use super::*;

    #[test]
    fn encrypt_decrypt() {
        let cq_cipher = CQCipher::new();
        let tq_cipher = TQCipher::new();
        let src = "Hello, World!";
        let mut dst = src.as_bytes().to_vec();
        cq_cipher.encrypt(&mut dst);
        tq_cipher.decrypt(&mut dst);
        let val = String::from_utf8_lossy(&dst);
        assert_eq!(src, &val);
        // ---
        let src = "Welcome";
        let mut dst = src.as_bytes().to_vec();
        tq_cipher.encrypt(&mut dst);
        cq_cipher.decrypt(&mut dst);
        let val = String::from_utf8_lossy(&dst);
        assert_eq!(src, &val);
    }

    #[test]
    fn encrypt_decrypt_with_generated_keys() {
        let cq_cipher = CQCipher::new();
        let tq_cipher = TQCipher::new();
        let src = [0u8; 28];
        let mut dst = src.to_vec();
        cq_cipher.encrypt(&mut dst);
        tq_cipher.decrypt(&mut dst);
        assert_eq!(src.as_slice(), &dst);
        // Exchange keys
        let seed: u64 = 0xc0ffeebabe;
        cq_cipher.generate_keys(seed);
        tq_cipher.generate_keys(seed);
        // --
        let src = [0u8; 52];
        let mut dst = src.to_vec();
        tq_cipher.encrypt(&mut dst);
        cq_cipher.decrypt(&mut dst);
        assert_eq!(src.as_slice(), &dst);
        // --
        let src = [0u8; 28];
        let mut dst = src.to_vec();
        cq_cipher.encrypt(&mut dst);
        tq_cipher.decrypt(&mut dst);
        assert_eq!(src.as_slice(), &dst);
    }
}
