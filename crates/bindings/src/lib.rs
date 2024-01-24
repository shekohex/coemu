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
    fn shutdown(actor: &Resource<tq_network::ActorHandle>);
    fn send(
        actor: &Resource<tq_network::ActorHandle>,
        packet_id: u16,
        packet_data_ptr: *const u8,
        packet_data_len: u32,
    );

    fn generate_login_token(
        actor: &Resource<tq_network::ActorHandle>,
        account_id: u32,
        realm_id: u32,
    ) -> u64;
}

/// Host bindings.
pub mod host {
    use crate::Resource;
    use tq_network::ActorHandle;

    pub use tracing_wasm::log;
    /// Shutdown an actor.
    pub fn shutdown(actor: &Resource<ActorHandle>) {
        unsafe { super::shutdown(actor) }
    }

    /// Send a packet to an actor.
    pub fn send<T: tq_network::PacketEncode>(
        actor: &Resource<ActorHandle>,
        packet: T,
    ) -> Result<(), T::Error> {
        let (packet_id, packet_data) = packet.encode()?;
        unsafe {
            super::send(
                actor,
                packet_id,
                packet_data.as_ptr(),
                packet_data.len() as u32,
            )
        }
        Ok(())
    }

    pub fn generate_login_token(
        actor: &Resource<ActorHandle>,
        account_id: u32,
        realm_id: u32,
    ) -> u64 {
        unsafe { super::generate_login_token(actor, account_id, realm_id) }
    }
}
