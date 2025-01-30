//! A Fixed Length String, used in Binary Packets
use core::fmt;
use core::marker::PhantomData;
use core::ops::Deref;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tq_crypto::{Cipher, TQRC5};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Fixed Length String.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FixedString<const N: usize, Mode> {
    inner: String,
    _mode: PhantomData<Mode>,
}

impl<T, const N: usize, M> From<T> for FixedString<N, M>
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        Self {
            inner: s.into(),
            _mode: PhantomData,
        }
    }
}

impl<const N: usize, M> Deref for FixedString<N, M> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<const N: usize, M> fmt::Display for FixedString<N, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}

impl<const N: usize> fmt::Debug for FixedString<N, ClearText> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixedString")
            .field("inner", &self.inner)
            .field("max_len", &N)
            .field("mode", &"clear_text")
            .finish()
    }
}

impl<const N: usize> fmt::Debug for FixedString<N, Encrypted> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixedString")
            .field("inner", &self.inner)
            .field("max_len", &N)
            .field("mode", &"encrypted")
            .finish()
    }
}

impl<const N: usize> fmt::Debug for FixedString<N, Masked> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FixedString")
            .field("inner", &self.inner)
            .field("max_len", &N)
            .field("mode", &"masked")
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearText;
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Encrypted;
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Masked;

impl<const N: usize> Serialize for FixedString<N, ClearText> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        encode_fixed_string::<N>(&self.inner).serialize(serializer)
    }
}

impl<const N: usize> Serialize for FixedString<N, Masked> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        encode_fixed_string::<N>(&self.inner).serialize(serializer)
    }
}

impl Serialize for FixedString<16, Encrypted> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut final_string = encode_fixed_string::<16>(&self.inner);
        let rc5 = TQRC5::new();
        rc5.encrypt(&mut final_string);
        final_string.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FixedString<10, ClearText> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let slice: [u8; 10] = Deserialize::deserialize(deserializer)?;
        let result = core::str::from_utf8(&slice).map_err(serde::de::Error::custom)?;
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}

impl<'de> Deserialize<'de> for FixedString<16, ClearText> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let slice: [u8; 16] = Deserialize::deserialize(deserializer)?;
        let result = core::str::from_utf8(&slice).map_err(serde::de::Error::custom)?;
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}

impl<'de> Deserialize<'de> for FixedString<16, Masked> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let slice: [u8; 16] = Deserialize::deserialize(deserializer)?;
        let result = core::str::from_utf8(&slice).map_err(serde::de::Error::custom)?;
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}

impl<'de> Deserialize<'de> for FixedString<16, Encrypted> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut slice: [u8; 16] = Deserialize::deserialize(deserializer)?;
        let rc5 = TQRC5::new();
        rc5.decrypt(&mut slice);
        let result = core::str::from_utf8(&slice).map_err(serde::de::Error::custom)?;
        let result = result.trim_end_matches('\0');
        Ok(result.into())
    }
}
/// Type Alias for a Fixed Length 16 Bytes String.
pub type String16 = FixedString<16, ClearText>;
/// Type Alias for a Fixed Length 10 Bytes String.
pub type String10 = FixedString<10, ClearText>;
/// Type Alias for a Fixed Length 16 Bytes Encrypted Password.
pub type TQPassword = FixedString<16, Encrypted>;
/// Type Alias for a Fixed Length 16 Bytes Masked Password.
/// A Masked Password is a Password that is not encrypted, but it is masked
/// with a * character.
pub type TQMaskedPassword = FixedString<16, Masked>;
