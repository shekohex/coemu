//! A Fixed Length String, used in Binary Packets
use core::fmt;
use crypto::{Cipher, TQRC5};
use encoding::{all::ASCII, DecoderTrap, EncoderTrap, Encoding};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{marker::PhantomData, ops::Deref};

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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

impl Serialize for FixedString<L16> {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut final_string = [0u8; 16];
        let mut string_encoded = ASCII
            .encode(&self.inner, EncoderTrap::Ignore)
            .expect("Never Fail");
        string_encoded.truncate(16);
        // Write value to final string
        for (i, c) in string_encoded.into_iter().enumerate() {
            final_string[i] = c;
        }
        final_string.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FixedString<L10> {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let slice: [u8; 10] = Deserialize::deserialize(deserializer)?;
        if slice.len() != 10 {
            return Err(serde::de::Error::custom(
                "input slice has wrong length",
            ));
        }
        let result = ASCII
            .decode(&slice, DecoderTrap::Ignore)
            .expect("Never Fails");
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}

impl<'de> Deserialize<'de> for FixedString<L16> {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let slice: [u8; 16] = Deserialize::deserialize(deserializer)?;
        if slice.len() != 16 {
            return Err(serde::de::Error::custom(
                "input slice has wrong length",
            ));
        }
        let result = ASCII
            .decode(&slice, DecoderTrap::Ignore)
            .expect("Never Fails");
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}

impl<'de> Deserialize<'de> for FixedString<EncryptedPassword> {
    fn deserialize<D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let slice: [u8; 16] = Deserialize::deserialize(deserializer)?;
        if slice.len() != 16 {
            return Err(serde::de::Error::custom(
                "input slice has wrong length",
            ));
        }
        let mut rc5 = TQRC5::new();
        let mut pass_decrypted_bytes = [0u8; 16];
        rc5.decrypt(&slice, &mut pass_decrypted_bytes);
        let result = ASCII
            .decode(&pass_decrypted_bytes, DecoderTrap::Ignore)
            .expect("Never Fails");
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
