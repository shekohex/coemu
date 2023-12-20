//! Externalities abstraction
//!
//! The externalities mainly provide access to storage and to registered
//! extensions. Extensions are for example the RNGs or the Http externalities.
//! These externalities are used to access the server from the runtime via the
//! runtime traits.
//!
//! This crate exposes the main [`Externalities`] trait.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

environmental::environmental!(ext: trait Externalities);

/// The Externalities.
///
/// Provides access to the storage and to other registered extensions.
pub trait Externalities {
    fn executer(&self) -> &sqlx::AnyPool;
}

/// Set the given externalities while executing the given closure. To get access
/// to the externalities while executing the given closure
/// [`with_externalities`] grants access to them. The externalities are only set
/// for the same thread this function was called from.
pub fn set_and_run_with_externalities<F, R>(
    ext: &mut dyn Externalities,
    f: F,
) -> R
where
    F: FnOnce() -> R,
{
    ext::using(ext, f)
}

/// Execute the given closure with the currently set externalities.
///
/// Returns `None` if no externalities are set or `Some(_)` with the result of
/// the closure.
pub fn with_externalities<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut dyn Externalities) -> R,
{
    ext::with(f)
}
