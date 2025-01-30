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

const PW32: u32 = 0xB7E15163;
const QW32: u32 = 0x61C88647;

const RC5_12: usize = 12;
const RC5_16: usize = 16;

const RC5_SUB: usize = RC5_12 * 2 + 2;
const RC5_KEY: usize = RC5_16 / 4;

const SEED: [u8; RC5_16] = [
    0x3C, 0xDC, 0xFE, 0xE8, 0xC4, 0x54, 0xD6, 0x7E, 0x16, 0xA6, 0xF8, 0x1A, 0xE8, 0xD0, 0x38, 0xBE,
];

/// Rivest Cipher 5 is implemented for interoperability with the Conquer Online
/// game client's login procedure. Passwords are encrypted in RC5 by the client,
/// and decrypted on the server to be hashed and compared to the database saved
/// password hash. In newer clients, this was replaced with SRP-6A (a hash based
/// exchange protocol).
#[derive(Copy, Clone)]
pub struct TQRC5 {
    rounds: usize,
    sub: [u32; 26],
}

impl TQRC5 {
    /// Initializes static variables for `RC5` to be interoperable with
    /// the Conquer Online game client.
    /// In later versions of the client, a random buffer is used to seed the
    /// cipher. This random buffer is sent to the client to establish a
    /// shared initial key.
    pub fn new() -> Self {
        let mut key = unsafe { core::mem::transmute::<[u8; RC5_16], [u32; RC5_KEY]>(SEED) };
        let mut sub = [0u32; RC5_SUB];
        sub[0] = PW32;

        for i in 1..RC5_SUB {
            sub[i] = sub[i - 1].wrapping_sub(QW32);
        }

        let mut i = 0;
        let mut j = 0;
        let mut x = 0;
        let mut y = 0;
        let count = core::cmp::max(RC5_SUB, RC5_KEY) * 3;

        for _ in 0..count {
            let value = sub[i].wrapping_add(x).wrapping_add(y);
            sub[i] = Self::rotl(value, 3);
            x = sub[i];
            i = (i + 1).wrapping_rem(RC5_SUB);
            let value = key[j].wrapping_add(x).wrapping_add(y);
            let count = x.wrapping_add(y);
            key[j] = Self::rotl(value, count);
            y = key[j];
            j = (j + 1).wrapping_rem(RC5_KEY);
        }

        Self { rounds: RC5_12, sub }
    }

    const fn rotl(value: u32, count: u32) -> u32 {
        let left_shift = count % 32;
        let right_shift = 32 - left_shift;
        value.wrapping_shl(left_shift) | value.wrapping_shr(right_shift)
    }

    const fn rotr(value: u32, count: u32) -> u32 {
        let right_shift = count % 32;
        let left_shift = 32 - right_shift;
        value.wrapping_shr(right_shift) | value.wrapping_shl(left_shift)
    }
}

impl Default for TQRC5 {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::Cipher for TQRC5 {
    fn generate_keys(&self, _seed: u64) {}

    fn decrypt(&self, data: &mut [u8]) {
        // Pad the buffer
        let mut src_len = data.len() / 8;
        if data.len() % 8 > 0 {
            src_len += 1;
        }

        for k in 0..src_len {
            let (lv_bytes, lf, lt) = {
                let (from, to) = (8 * k, (8 * k) + 4);
                (&data[from..to].try_into().unwrap(), from, to)
            };
            let (rv_bytes, rf, rt) = {
                let (from, to) = (lt, lt + 4);
                (&data[from..to].try_into().unwrap(), from, to)
            };
            let mut lv = u32::from_le_bytes(*lv_bytes);
            let mut rv = u32::from_le_bytes(*rv_bytes);

            for i in (1..=self.rounds).rev() {
                rv = Self::rotr(rv.wrapping_sub(self.sub[2 * i + 1]), lv) ^ lv;
                lv = Self::rotr(lv.wrapping_sub(self.sub[2 * i]), rv) ^ rv;
            }

            lv = lv.wrapping_sub(self.sub[0]);
            rv = rv.wrapping_sub(self.sub[1]);
            data[lf..lt].copy_from_slice(&lv.to_le_bytes());
            data[rf..rt].copy_from_slice(&rv.to_le_bytes());
        }
    }

    fn encrypt(&self, data: &mut [u8]) {
        let mut src_len = data.len() / 8;
        if data.len() % 8 > 0 {
            src_len += 1;
        }

        for k in 0..src_len {
            let (lv_bytes, lf, lt) = {
                let (from, to) = (8 * k, (8 * k) + 4);
                (&data[from..to].try_into().unwrap(), from, to)
            };
            let (rv_bytes, rf, rt) = {
                let (from, to) = (lt, lt + 4);
                (&data[from..to].try_into().unwrap(), from, to)
            };
            let mut lv = u32::from_le_bytes(*lv_bytes).wrapping_add(self.sub[0]);
            let mut rv = u32::from_le_bytes(*rv_bytes).wrapping_add(self.sub[1]);

            for i in 1..=self.rounds {
                lv = Self::rotl(lv ^ rv, rv).wrapping_add(self.sub[2 * i]);
                rv = Self::rotl(rv ^ lv, lv).wrapping_add(self.sub[2 * i + 1]);
            }

            data[lf..lt].copy_from_slice(&lv.to_le_bytes());
            data[rf..rt].copy_from_slice(&rv.to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TQRC5;
    use crate::Cipher;
    #[test]
    fn seed() {
        const EXPECTED_SUB_KEY_SEED: [u32; 26] = [
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

        let rc5 = TQRC5::new();
        assert_eq!(rc5.sub, EXPECTED_SUB_KEY_SEED);
    }

    #[test]
    fn rc5() {
        let rc5 = TQRC5::new();
        let mut buf = [
            0x1C, 0xFD, 0x41, 0xC9, 0xA1, 0x69, 0xAA, 0xB6, 0x0D, 0xA6, 0x08, 0x4D, 0xF3, 0x67, 0xEB, 0x73,
        ];
        rc5.decrypt(&mut buf);
        assert_eq!(
            buf,
            [0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn encrypt_decrypt() {
        let rc5 = TQRC5::new();
        let mut buf = [0x31; 16];
        let original = buf;
        rc5.encrypt(&mut buf);
        rc5.decrypt(&mut buf);
        assert_eq!(buf, origional);
    }
}
