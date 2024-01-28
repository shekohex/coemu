//! Wasm-specific extensions for `tracing`.
//! This crate provides a `tracing` layer for use in WASM environments. It
//! provides a `tracing` layer for use in WASM environments, and is used by
//! all packets that needs to emit tracing events.
//!
//! # Usage
//! ```rust, no_run
//! use tracing_wasm::MakeWasmWriter;
//! use tracing_subscriber::prelude::*;
//!
//! let fmt_layer = tracing_subscriber::fmt::layer()
//!     .without_time()   // std::time is not available in host
//!     .with_writer(MakeWasmWriter::new()); // write events to the host
//! tracing_subscriber::registry()
//!     .with(fmt_layer)
//!     .init();
//! ```
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

pub use tracing_core::Level;

#[cfg(feature = "std")]
use tracing_subscriber::fmt::MakeWriter;

#[link(wasm_import_module = "host")]
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn trace_event(level: u8, target: *const u8, target_len: u32, message: *const u8, message_len: u32);
}

/// A [`MakeWriter`] emitting the written text to the [`host`].
pub struct MakeWasmWriter {
    use_pretty_label: bool,
    target: &'static str,
}

impl Default for MakeWasmWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl MakeWasmWriter {
    /// Create a default console writer, i.e. no level annotation is shown when
    /// logging a message.
    pub fn new() -> Self {
        Self {
            use_pretty_label: false,
            target: "wasm",
        }
    }

    /// Change writer with the given target.
    pub fn with_target(mut self, target: &'static str) -> Self {
        self.target = target;
        self
    }

    /// Enables an additional label for the log level to be shown.
    ///
    /// It is recommended that you also use [`Layer::with_level(false)`] if you
    /// use this option, to avoid the event level being shown twice.
    ///
    /// [`Layer::with_level(false)`]: tracing_subscriber::fmt::Layer::with_level
    pub fn with_pretty_level(mut self) -> Self {
        self.use_pretty_label = true;
        self
    }
}

type LogDispatcher = fn(Level, &str, &str);

/// Dispatches a log message to the host.
#[cfg(target_arch = "wasm32")]
pub fn log(level: Level, target: &str, message: &str) {
    let level = match level {
        Level::ERROR => 0,
        Level::WARN => 1,
        Level::INFO => 2,
        Level::DEBUG => 3,
        Level::TRACE => 4,
    };
    let message = message.as_bytes();
    let target = target.as_bytes();
    unsafe {
        trace_event(
            level,
            target.as_ptr(),
            target.len() as u32,
            message.as_ptr(),
            message.len() as u32,
        )
    }
}

/// Does nothing.

#[cfg(not(target_arch = "wasm32"))]
pub fn log(_level: Level, _target: &str, _message: &str) {}

/// Concrete [`std::io::Write`] implementation returned by [`MakeWasmWriter`].
pub struct WasmWriter {
    buffer: Vec<u8>,
    target: String,
    level: Level,
    log: LogDispatcher,
}

impl Drop for WasmWriter {
    fn drop(&mut self) {
        let message = String::from_utf8_lossy(&self.buffer);
        (self.log)(self.level, self.target.as_ref(), message.as_ref())
    }
}

#[cfg(feature = "std")]
impl std::io::Write for WasmWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
#[cfg(feature = "std")]
impl<'a> MakeWriter<'a> for MakeWasmWriter {
    type Writer = WasmWriter;

    fn make_writer(&'a self) -> Self::Writer {
        WasmWriter {
            buffer: Vec::new(),
            level: Level::TRACE,
            target: self.target.to_string(),
            log,
        }
    }

    fn make_writer_for(&'a self, meta: &tracing_core::Metadata<'_>) -> Self::Writer {
        let level = *meta.level();
        let target = meta.target().to_string();
        WasmWriter {
            buffer: Vec::new(),
            target,
            level,
            log,
        }
    }
}
