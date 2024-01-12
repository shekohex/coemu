//! Bindings to the host environment.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub use externref::{self as anyref, externref, Resource};

/// Generate host bindings.
#[macro_export]
macro_rules! generate {
    () => {
        #[$crate::externref(crate = "tq_bindings::anyref")]
        #[link(wasm_import_module = "host")]
        extern "C" {
            fn shutdown(actor: &Resource<tq_network::ActorHandle>);
        }

        /// Host bindings.
        mod host {
            use $crate::Resource;
            use tq_network::ActorHandle;
            /// Shutdown an actor.
            pub fn shutdown(actor: &Resource<ActorHandle>) {
                unsafe { super::shutdown(actor) }
            }
        }
    };
}
