//! RC5 is implemented in the Account Server for password verification. It was
//! removed around patch 5528, but can be reimplemented using hooks. Keys found
//! in this implementation are targeted for the Conquer Online game client. This
//! implementation was programmed by [CptSky][1] in his [COPS v6][2]
//! project.
//!
//! [1]: https://www.elitepvpers.com/forum/members/568265-cptsky.html
//! [2]: https://www.elitepvpers.com/forum/co2-pserver-guides-releases/2402439-cops-v6-source-tools-custom-emulator.html
//!
//! RC5 is a symmetric-key block cipher notable for its simplicity. Designed by
//! Ronald Rivest in 1994.
//!
//! Unlike many schemes, RC5 has a variable block size (32, 64 or 128 bits), key
//! size (0 to 2040 bits) and number of rounds (0 to 255). The original
//! suggested choice of parameters were a block size of 64 bits, a 128-bit key
//! and 12 rounds.
//!
//! A key feature of RC5 is the use of data-dependent rotations. RC5 also
//! consists of a number of modular additions and eXclusive OR (XOR)s. The
//! general structure of the algorithm is a Feistel-like network.

use bytes::{Buf, BufMut};

const SUB_KEY_SEED: [u32; 26] = [
    0xA991_5556,
    0x48E4_4110,
    0x9F32_308F,
    0x27F4_1D3E,
    0xCF4F_3523,
    0xEAC3_C6B4,
    0xE9EA_5E03,
    0xE597_4BBA,
    0x334D_7692,
    0x2C6B_CF2E,
    0x0DC5_3B74,
    0x995C_92A6,
    0x7E4F_6D77,
    0x1EB2_B79F,
    0x1D34_8D89,
    0xED64_1354,
    0x15E0_4A9D,
    0x488D_A159,
    0x6478_17D3,
    0x8CA0_BC20,
    0x9264_F7FE,
    0x91E7_8C6C,
    0x5C9A_07FB,
    0xABD4_DCCE,
    0x6416_F98D,
    0x6642_AB5B,
];

/// Rivest Cipher 5 is implemented for interoperability with the Conquer Online
/// game client's login procedure. Passwords are encrypted in RC5 by the client,
/// and decrypted on the server to be hashed and compared to the database saved
/// password hash. In newer clients, this was replaced with SRP-6A (a hash based
/// exchange protocol).
#[derive(Copy, Clone)]
pub struct TQRC5 {
    rounds: u8,
    sub: [u32; 26],
}

impl TQRC5 {
    /// Initializes static variables for `RC5` to be interoperable with
    /// the Conquer Online game client.
    /// In later versions of the client, a random buffer is used to seed the
    /// cipher. This random buffer is sent to the client to establish a
    /// shared initial key.
    pub const fn new() -> Self {
        let rounds = 12;
        Self {
            rounds,
            sub: SUB_KEY_SEED,
        }
    }
}

impl Default for TQRC5 {
    fn default() -> Self { Self::new() }
}

impl crate::Cipher for TQRC5 {
    fn generate_keys(&self, _seed: u64) {}

    fn decrypt(&self, data: &mut [u8]) {
        // Pad the buffer
        let mut src_len = data.len() / 8;
        if data.len() % 8 > 0 {
            src_len += 1;
        }
        // Decrypt the buffer
        for word in 0..src_len {
            let mut chunk_a = &data[8 * word..];
            let mut chunk_b = &data[(8 * word + 4)..];
            let mut a = chunk_a.get_u32_le();
            let mut b = chunk_b.get_u32_le();
            let rounds = self.rounds;
            let sub = self.sub;

            for round in (1..=rounds).rev() {
                b = (b.wrapping_sub(sub[(2 * round + 1) as usize]))
                    .rotate_right(a)
                    ^ a;
                a = (a.wrapping_sub(sub[(2 * round) as usize])).rotate_right(b)
                    ^ b;
            }
            let chunk_a = &mut data[(8 * word)..];
            let mut wtr_a = vec![];
            wtr_a.put_u32_le(a.wrapping_sub(sub[0]));
            for (i, b) in wtr_a.iter().enumerate() {
                chunk_a[i] = *b;
            }
            let chunk_b = &mut data[(8 * word + 4)..];
            let mut wtr_b = vec![];
            wtr_b.put_u32_le(b.wrapping_sub(sub[1]));

            for (i, b) in wtr_b.iter().enumerate() {
                chunk_b[i] = *b;
            }
        }
    }

    fn encrypt(&self, _data: &mut [u8]) {
        unimplemented!("RC5 encryption is not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::TQRC5;
    use crate::Cipher;
    #[test]
    fn test_rc5() {
        let rc5 = TQRC5::new();
        let mut buf = [
            0x1C, 0xFD, 0x41, 0xC9, 0xA1, 0x69, 0xAA, 0xB6, 0x0D, 0xA6, 0x08,
            0x4D, 0xF3, 0x67, 0xEB, 0x73,
        ];
        rc5.decrypt(&mut buf);
        assert_eq!(
            buf,
            [
                0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ]
        );
    }
}
