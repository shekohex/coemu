//! Defines a type that serializes to a list of strings.
//!
//! A StringList is effectively a `Vec<String>` that serializes to a list of
//! strings which is prefixed by the number of strings in the list as the first
//! byte then followed by the length of each string as a byte followed by the
//! string itself encoded in UTF-8.
//!
//! # Examples
//! ```no_run
//! use tq_serde::StringList;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct MyStruct {
//!   my_list: StringList,
//! }
//!
//! let my_struct = MyStruct {
//!  my_list: StringList::from(vec!["Hello", "World"]),
//! };
//!
//! let bytes = tq_serde::to_bytes(&my_struct).unwrap();
//! ```
//!
//! # Notes
//!
//! The maximum number of strings in the list is 255 and the maximum length of
//! each string is 250 bytes.

use bytes::Buf;

#[cfg(feature = "std")]
use std::vec;

#[cfg(not(feature = "std"))]
use alloc::{
    string::{String, ToString},
    vec::{self, Vec},
};

/// Defines a type that serializes to a list of strings.
///
/// Read the [module level documentation](index.html) for more information.
#[derive(Default, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub struct StringList {
    inner: Vec<String>,
}

impl core::fmt::Debug for StringList {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.inner.is_empty() {
            return write!(f, "[]");
        }
        if self.inner.len() == 1 {
            return write!(f, "[{:?}]", self.inner[0]);
        }
        write!(f, "[")?;
        for (i, s) in self.inner.iter().enumerate() {
            if i != 0 || i != self.inner.len() - 1 {
                write!(f, ", ")?;
            }
            write!(f, "{:?}", s)?;
        }
        write!(f, "]")
    }
}

impl StringList {
    /// Creates a new empty StringList.
    pub fn new() -> Self {
        StringList { inner: Vec::new() }
    }

    /// Pushes a new string onto the StringList.
    pub fn push(&mut self, s: String) {
        self.inner.push(s);
    }

    /// Returns the number of strings in the StringList.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the StringList is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns a reference to the Vec<String> that the StringList wraps.
    pub fn as_vec(&self) -> &Vec<String> {
        &self.inner
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.inner.iter()
    }
}

impl<T> From<Vec<T>> for StringList
where
    T: Into<String>,
{
    fn from(strings: Vec<T>) -> Self {
        StringList::from_iter(strings)
    }
}

impl From<StringList> for Vec<String> {
    fn from(string_list: StringList) -> Self {
        string_list.inner
    }
}

impl AsRef<Vec<String>> for StringList {
    fn as_ref(&self) -> &Vec<String> {
        &self.inner
    }
}

impl AsRef<[String]> for StringList {
    fn as_ref(&self) -> &[String] {
        &self.inner
    }
}

impl AsMut<Vec<String>> for StringList {
    fn as_mut(&mut self) -> &mut Vec<String> {
        &mut self.inner
    }
}

impl AsMut<[String]> for StringList {
    fn as_mut(&mut self) -> &mut [String] {
        &mut self.inner
    }
}

impl<T: Into<String>> FromIterator<T> for StringList {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        StringList {
            inner: iter.into_iter().take((u8::MAX - 1) as _).map(|s| s.into()).collect(),
        }
    }
}

impl IntoIterator for StringList {
    type IntoIter = vec::IntoIter<String>;
    type Item = String;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl serde::Serialize for StringList {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = self.inner.len() as u8;
        let total_string_len = self.inner.iter().map(|s| s.len() + 1).sum::<usize>();
        let mut bytes = Vec::with_capacity(total_string_len);
        bytes.push(len);
        for s in &self.inner {
            bytes.push(s.len() as u8);
            for c in s.chars() {
                // remove any non-ascii char and replace it with '?' instead
                // Write value to final string
                if c.is_ascii() {
                    bytes.push(c as u8);
                } else {
                    bytes.push(b'?');
                }
            }
        }
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> serde::Deserialize<'de> for StringList {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StringListVisitor;

        impl<'de> serde::de::Visitor<'de> for StringListVisitor {
            type Value = StringList;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a list of strings")
            }

            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                let mut strings = Vec::new();
                let mut reader = bytes::Bytes::copy_from_slice(v);
                let len = reader.get_u8() as usize;
                for _ in 0..len {
                    let string_len = reader.get_u8() as usize;
                    let string_bytes = reader.copy_to_bytes(string_len);
                    let string = core::str::from_utf8(&string_bytes)
                        .map(|s| s.trim_end_matches('\0'))
                        .map_err(serde::de::Error::custom)?;
                    strings.push(string.to_string());
                }
                Ok(StringList { inner: strings })
            }
        }

        deserializer.deserialize_bytes(StringListVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[test]
    fn test_new() {
        let list = StringList::new();
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_push() {
        let mut list = StringList::new();
        list.push("hello".to_string());
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());
        assert_eq!(list.as_vec()[0], "hello");
    }

    #[test]
    fn test_from_vec() {
        let vec = vec!["hello".to_string(), "world".to_string()];
        let list = StringList::from(vec.clone());
        assert_eq!(list.len(), 2);
        assert_eq!(list.as_vec(), &vec);
    }

    #[test]
    fn test_iter() {
        let vec = vec!["hello".to_string(), "world".to_string()];
        let list = StringList::from_iter(vec.clone());
        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&"hello".to_string()));
        assert_eq!(iter.next(), Some(&"world".to_string()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_from_iterator() {
        let list: StringList = ["hello", "world"].iter().map(|s| s.to_string()).collect();
        assert_eq!(list.len(), 2);
        assert_eq!(list.as_vec()[0], "hello");
        assert_eq!(list.as_vec()[1], "world");
    }

    #[test]
    fn test_serialize_deserialize() {
        let list = StringList::from(vec!["hello".to_string(), "world".to_string()]);
        let serialized = crate::to_bytes(&list).unwrap();
        let deserialized: StringList = crate::from_bytes(&serialized).unwrap();
        assert_eq!(list, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_msg() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub struct MsgTaskDialog {
            task_id: u32,
            avatar: u16,
            option_id: u8,
            action: u8,
            msgs: StringList,
        }
        let msg = MsgTaskDialog {
            task_id: 1,
            avatar: 2,
            option_id: 3,
            action: 4,
            msgs: StringList::from(vec!["hello", "world"]),
        };
        let serialized = crate::to_bytes(&msg).unwrap();
        let deserialized: MsgTaskDialog = crate::from_bytes(&serialized).unwrap();
        assert_eq!(msg, deserialized);
    }
}
