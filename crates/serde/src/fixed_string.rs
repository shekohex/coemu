//! A Fixed Length String, used in Binary Packets
use core::fmt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::marker::PhantomData;
use std::ops::Deref;
use tq_crypto::{Cipher, TQRC5};

/// A Marker Trait for a Fixed Length String.
pub trait FixedLen {}

/// Fixed Length 16 Bytes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct L16;

/// Fixed Length 10 Bytes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct L10;

/// Fixed Length 16 Encrypted Bytes.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct EncryptedPassword;

impl FixedLen for L16 {}
impl FixedLen for L10 {}
impl FixedLen for EncryptedPassword {}

/// Fixed Length String.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FixedString<L: FixedLen> {
    inner: String,
    __len: PhantomData<L>,
}

impl<T, L> From<T> for FixedString<L>
where
    T: Into<String>,
    L: FixedLen,
{
    fn from(s: T) -> Self {
        Self {
            inner: s.into(),
            __len: PhantomData,
        }
    }
}

impl<L: FixedLen> Deref for FixedString<L> {
    type Target = str;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl<L: FixedLen> fmt::Display for FixedString<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl<L: FixedLen> fmt::Debug for FixedString<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixedString")
            .field("inner", &self.inner)
            .finish()
    }
}

fn encode_fixed_string<const N: usize>(s: &str) -> [u8; N] {
    let mut final_string = [0u8; N];
    let string_encoded = s.as_bytes();
    // remove any non-ascii char and replace it with '?' instead
    // Write value to final string
    for (i, b) in string_encoded.iter().take(N).enumerate() {
        if b.is_ascii() {
            final_string[i] = *b;
        } else {
            final_string[i] = b'?';
        }
    }
    final_string
}

impl Serialize for FixedString<L16> {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        encode_fixed_string::<16>(&self.inner).serialize(serializer)
    }
}

impl Serialize for FixedString<L10> {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        encode_fixed_string::<10>(&self.inner).serialize(serializer)
    }
}

impl Serialize for FixedString<EncryptedPassword> {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        // FIXME: encrypt password
        encode_fixed_string::<16>(&self.inner).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FixedString<L10> {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let slice: [u8; 10] = Deserialize::deserialize(deserializer)?;
        let result =
            std::str::from_utf8(&slice).map_err(serde::de::Error::custom)?;
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}

impl<'de> Deserialize<'de> for FixedString<L16> {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let slice: [u8; 16] = Deserialize::deserialize(deserializer)?;
        let result =
            std::str::from_utf8(&slice).map_err(serde::de::Error::custom)?;
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}

impl<'de> Deserialize<'de> for FixedString<EncryptedPassword> {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let slice: [u8; 16] = Deserialize::deserialize(deserializer)?;
        let rc5 = TQRC5::new();
        let mut pass_decrypted_bytes = [0u8; 16];
        rc5.decrypt(&slice, &mut pass_decrypted_bytes);
        let result = std::str::from_utf8(&pass_decrypted_bytes)
            .map_err(serde::de::Error::custom)?;
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}
/// Type Alias for a Fixed Length 16 Bytes String.
pub type String16 = FixedString<L16>;
/// Type Alias for a Fixed Length 10 Bytes String.
pub type String10 = FixedString<L10>;
/// Type Alias for a Fixed Length 16 Bytes Encrypted Password.
pub type TQPassword = FixedString<EncryptedPassword>;
