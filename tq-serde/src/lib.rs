//! This Crate is used to create a Binary Serialization and Deserialization on
//! top of [serde](https://serde.rs).
//! It will be use to Serialize and Deserialize Conquer Online Binary Packets.
mod errors;
pub use errors::TQSerdeError;

mod fixed_string;
pub use fixed_string::{String10, String16, TQPassword};

mod ser;
pub use ser::to_bytes;

mod de;
pub use de::from_bytes;
