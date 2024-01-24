//! Bindings to the host environment.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub use externref::{self as anyref, externref, Resource};

/// A [`MakeWriter`] emitting the written text to the [`host`].
#[cfg(feature = "std")]
pub fn setup_logging(name: &'static str) {
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_level(false)
        .with_target(false)
        .with_max_level(tracing_wasm::Level::TRACE)
        .with_writer(tracing_wasm::MakeWasmWriter::new().with_target(name))
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

/// A [`MakeWriter`] emitting the written text to the [`host`].
#[cfg(not(feature = "std"))]
pub fn setup_logging(_name: &'static str) {}

/// Sets a panic hook that logs to the host.
#[cfg(feature = "std")]
pub fn set_panic_hook_once(name: &'static str) {
    static SET_HOOK: std::sync::Once = std::sync::Once::new();
    SET_HOOK.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            let payload = info
                .payload()
                .downcast_ref::<&str>()
                .copied()
                .unwrap_or_else(|| {
                    info.payload().downcast_ref::<String>().unwrap().as_str()
                });
            let location = info
                .location()
                .map(|l| format!("{}:{}", l.file(), l.line()));
            host::log(
                tracing_wasm::Level::ERROR,
                name,
                &format!("'{payload}' at {}", location.unwrap_or_default()),
            );
        }));
    });
}

#[cfg(not(feature = "std"))]
pub fn set_panic_hook_once(_name: &'static str) {}

#[externref(crate = "crate::anyref")]
#[link(wasm_import_module = "host")]
extern "C" {
    fn getrandom(ptr: *mut u8, len: usize) -> i32;
    fn tq_network_actor_shutdown(actor: &Resource<tq_network::ActorHandle>);
    fn tq_network_actor_send(
        actor: &Resource<tq_network::ActorHandle>,
        packet_id: u16,
        packet_data_ptr: *const u8,
        packet_data_len: u32,
    );

    fn tq_db_realm_by_name(
        realm_name_ptr: *const u8,
        realm_name_len: u32,
        out_realm_ptr: *mut u8,
        out_realm_len: *mut u32,
    ) -> i32;

    fn game_state_generate_login_token(
        actor: &Resource<tq_network::ActorHandle>,
        account_id: u32,
        realm_id: u32,
    ) -> u64;
}

/// Host bindings.
pub mod host {
    /// [`tracing_wasm`] bindings.
    pub use tracing_wasm::log;
    /// [`tq_network`] bindings.
    pub mod network {
        /// [`tq_network::actor`] bindings.
        pub mod actor {
            use crate::Resource;
            use tq_network::ActorHandle;
            /// [`tq_network::actor::ActorHandle::shutdown`] bindings.
            pub fn shutdown(actor: &Resource<ActorHandle>) {
                unsafe { crate::tq_network_actor_shutdown(actor) }
            }
            /// [`tq_network::actor::ActorHandle::send`] bindings.
            pub fn send<T: tq_network::PacketEncode>(
                actor: &Resource<ActorHandle>,
                packet: T,
            ) -> Result<(), T::Error> {
                let (packet_id, packet_data) = packet.encode()?;
                unsafe {
                    crate::tq_network_actor_send(
                        actor,
                        packet_id,
                        packet_data.as_ptr(),
                        packet_data.len() as u32,
                    )
                }
                Ok(())
            }
        }
    }

    /// [`tq_db`] bindings.
    pub mod db {
        /// [`tq_db::realm`] bindings.
        pub mod realm {
            use tq_db::realm::Realm;
            /// [`tq_db::realm::Realm::by_name`] bindings.
            pub fn by_name(
                realm_name: &str,
            ) -> Result<Option<Realm>, tq_db::Error> {
                let realm = core::ptr::null_mut();
                let mut realm_len = 0;
                let res = unsafe {
                    crate::tq_db_realm_by_name(
                        realm_name.as_ptr(),
                        realm_name.len() as u32,
                        realm,
                        &mut realm_len,
                    )
                };
                if res == 0 && realm_len > 0 && !realm.is_null() {
                    let realm = unsafe {
                        let bytes = std::vec::Vec::from_raw_parts(
                            realm,
                            realm_len as usize,
                            realm_len as usize,
                        );
                        // Does not work!
                        core::mem::transmute(bytes)
                    };
                    Ok(Some(realm))
                } else {
                    Ok(None)
                }
            }
        }
    }
    /// [`game`] bindings.
    pub mod game {
        /// [`game::state`] bindings.
        pub mod state {
            use crate::Resource;
            use tq_network::ActorHandle;
            /// [`game::state::generate_login_token`] bindings.
            pub fn generate_login_token(
                actor: &Resource<ActorHandle>,
                account_id: u32,
                realm_id: u32,
            ) -> u64 {
                unsafe {
                    crate::game_state_generate_login_token(
                        actor, account_id, realm_id,
                    )
                }
            }
        }
    }

    /// Get random bytes.
    pub fn getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
        let res = unsafe { super::getrandom(buf.as_mut_ptr(), buf.len()) };
        if res == 0 {
            Ok(())
        } else {
            Err(getrandom::Error::FAILED_RDRAND)
        }
    }
}

getrandom::register_custom_getrandom!(host::getrandom);
