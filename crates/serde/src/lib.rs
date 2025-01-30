//! This Crate is used to create a Binary Serialization and Deserialization on
//! top of [serde](https://serde.rs).
//! It will be use to Serialize and Deserialize Conquer Online Binary Packets.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

mod errors;
pub use errors::TQSerdeError;

mod fixed_string;
pub use fixed_string::{String10, String16, TQMaskedPassword, TQPassword};

mod string_list;
pub use string_list::StringList;

mod ser;
pub use ser::to_bytes;

mod de;
pub use de::from_bytes;
