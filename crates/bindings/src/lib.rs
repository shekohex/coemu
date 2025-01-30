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

pub fn encode_ptr_len(a: *mut u8, b: usize) -> u64 {
    (a as u64) << 32 | b as u64
}

pub fn decode_ptr_len(c: u64) -> (*mut u8, usize) {
    ((c >> 32) as u32 as *mut u8, c as u32 as usize)
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
                .unwrap_or_else(|| info.payload().downcast_ref::<String>().unwrap().as_str());
            let location = info.location().map(|l| format!("{}:{}", l.file(), l.line()));
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
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn getrandom(buffer: u64) -> i32;
    fn tq_network_actor_shutdown(actor: &Resource<tq_network::ActorHandle>);
    fn tq_network_actor_send(actor: &Resource<tq_network::ActorHandle>, packet_id: u16, packet: u64);

    fn tq_network_actor_set_id(actor: &Resource<tq_network::ActorHandle>, id: u32);

    fn tq_db_realm_by_name(realm_name: u64) -> u64;

    fn tq_db_account_auth(username: u64, password: u64) -> i32;

    fn game_state_generate_login_token(
        actor: &Resource<tq_network::ActorHandle>,
        account_id: u32,
        realm_id: u32,
    ) -> u64;

    fn auth_server_bus_check(realm: u64) -> i32;
    fn auth_server_bus_transfer(actor: &Resource<tq_network::ActorHandle>, realm: u64) -> u64;
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
            #[cfg(target_arch = "wasm32")]
            pub fn shutdown(actor: &Resource<ActorHandle>) {
                unsafe { crate::tq_network_actor_shutdown(actor) }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn shutdown(_actor: &Resource<ActorHandle>) {}

            /// [`tq_network::actor::ActorHandle::send`] bindings.
            #[cfg(target_arch = "wasm32")]
            pub fn send<T: tq_network::PacketEncode>(actor: &Resource<ActorHandle>, packet: T) -> Result<(), T::Error> {
                let (packet_id, packet_data) = packet.encode()?;
                let packet = crate::encode_ptr_len(packet_data.as_ptr() as *mut u8, packet_data.len());
                unsafe { crate::tq_network_actor_send(actor, packet_id, packet) }
                Ok(())
            }
            #[cfg(not(target_arch = "wasm32"))]
            pub fn send<T: tq_network::PacketEncode>(
                _actor: &Resource<ActorHandle>,
                _packet: T,
            ) -> Result<(), T::Error> {
                Ok(())
            }
            /// [`tq_network::actor::ActorHandle::set_id`] bindings.
            #[cfg(target_arch = "wasm32")]
            pub fn set_id(actor: &Resource<ActorHandle>, id: u32) {
                unsafe { crate::tq_network_actor_set_id(actor, id) }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn set_id(_actor: &Resource<ActorHandle>, _id: u32) {}
        }
    }

    /// [`tq_db`] bindings.
    pub mod db {
        /// [`tq_db::account`] bindings.
        pub mod account {

            /// [`tq_db::account::Account::auth`] bindings.
            #[cfg(target_arch = "wasm32")]
            pub fn auth(username: &str, password: &str) -> Result<u32, tq_db::Error> {
                let res = unsafe {
                    let username = crate::encode_ptr_len(username.as_ptr() as *mut u8, username.len());
                    let password = crate::encode_ptr_len(password.as_ptr() as *mut u8, password.len());
                    crate::tq_db_account_auth(username, password)
                };
                if res > 0 {
                    Ok(res as u32)
                } else {
                    match res {
                        -1 => Err(tq_db::Error::AccountNotFound),
                        -2 => Err(tq_db::Error::InvalidPassword),
                        _ => unreachable!("Unknown error code"),
                    }
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn auth(_username: &str, _password: &str) -> Result<u32, tq_db::Error> {
                unimplemented!("Not implemented on non-wasm32")
            }
        }

        /// [`tq_db::realm`] bindings.
        pub mod realm {
            use tq_db::realm::Realm;
            /// [`tq_db::realm::Realm::by_name`] bindings.
            #[cfg(target_arch = "wasm32")]
            pub fn by_name(realm_name: &str) -> Result<Option<Realm>, tq_db::Error> {
                use rkyv::Deserialize;
                let realm_encoded = unsafe {
                    let realm_name = crate::encode_ptr_len(realm_name.as_ptr() as *mut u8, realm_name.len());
                    crate::tq_db_realm_by_name(realm_name)
                };
                let (realm_ptr, realm_len) = crate::decode_ptr_len(realm_encoded);
                if !realm_ptr.is_null() && realm_len > 0 {
                    let realm = unsafe {
                        let bytes = std::vec::Vec::from_raw_parts(realm_ptr, realm_len, realm_len);
                        let archived = rkyv::archived_root::<Realm>(&bytes);
                        archived.deserialize(&mut rkyv::Infallible).unwrap()
                    };
                    Ok(Some(realm))
                } else {
                    Ok(None)
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn by_name(_realm_name: &str) -> Result<Option<Realm>, tq_db::Error> {
                unimplemented!("Not implemented on non-wasm32")
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
            #[cfg(target_arch = "wasm32")]
            pub fn generate_login_token(actor: &Resource<ActorHandle>, account_id: u32, realm_id: u32) -> u64 {
                unsafe { crate::game_state_generate_login_token(actor, account_id, realm_id) }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn generate_login_token(_actor: &Resource<ActorHandle>, _account_id: u32, _realm_id: u32) -> u64 {
                unimplemented!("Not implemented on non-wasm32")
            }
        }
    }

    /// [`auth`] bindings.
    pub mod auth {
        /// [`auth::server_bus`] bindings.
        pub mod server_bus {
            use externref::Resource;
            use tq_db::realm::Realm;

            /// [`auth::server_bus::check`] bindings.
            #[cfg(target_arch = "wasm32")]
            pub fn check(realm: &Realm) -> Result<(), tq_network::Error> {
                let archived = rkyv::to_bytes::<_, 64>(realm).unwrap();
                let res = unsafe {
                    let archived = crate::encode_ptr_len(archived.as_ptr() as *mut u8, archived.len());
                    crate::auth_server_bus_check(archived)
                };
                if res == 0 {
                    Ok(())
                } else {
                    Err(tq_network::Error::Other(String::from("Server Down")))
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn check(_realm: &Realm) -> Result<(), tq_network::Error> {
                unimplemented!("Not implemented on non-wasm32")
            }

            /// [`auth::server_bus::transfer`] bindings.
            #[cfg(target_arch = "wasm32")]
            pub fn transfer(
                actor: &Resource<tq_network::ActorHandle>,
                realm: &Realm,
            ) -> Result<u64, tq_network::Error> {
                let archived = rkyv::to_bytes::<_, 64>(realm).unwrap();
                let token = unsafe {
                    let archived = crate::encode_ptr_len(archived.as_ptr() as *mut u8, archived.len());
                    crate::auth_server_bus_transfer(actor, archived)
                };
                if token == 0 {
                    Err(tq_network::Error::Other(String::from("Server Timed Out")))
                } else {
                    Ok(token)
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            pub fn transfer(
                _actor: &Resource<tq_network::ActorHandle>,
                _realm: &Realm,
            ) -> Result<u64, tq_network::Error> {
                unimplemented!("Not implemented on non-wasm32")
            }
        }
    }

    /// Get random bytes.
    #[cfg(target_arch = "wasm32")]
    pub fn getrandom(buf: &mut [u8]) -> Result<(), getrandom::Error> {
        let res = unsafe {
            let buffer = crate::encode_ptr_len(buf.as_mut_ptr(), buf.len());
            super::getrandom(buffer)
        };
        if res == 0 {
            Ok(())
        } else {
            Err(getrandom::Error::FAILED_RDRAND)
        }
    }
}

#[cfg(target_arch = "wasm32")]
getrandom::register_custom_getrandom!(host::getrandom);
