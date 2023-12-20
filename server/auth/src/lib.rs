//! Auth Server

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod error;
pub mod state;

pub use error::Error;
pub use state::State;
