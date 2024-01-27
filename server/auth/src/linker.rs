use wasmtime::Linker;

const MODULE: &str = "host";

/// Read a slice of bytes from wasm memory.
macro_rules! memof {
    ($caller:expr) => {
        $caller
            .get_export("memory")
            .and_then(::wasmtime::Extern::into_memory)
            .expect("failed to read wasm memory")
    };
}

macro_rules! mread {
    ($caller:expr, $mem:expr, $ptr:expr, $size:expr) => {{
        $mem.data(&$caller)
            .get($ptr as usize..)
            .and_then(|s| s.get(..$size as usize))
            .expect("failed to read wasm memory")
    }};
}

macro_rules! mread_mut {
    ($caller:expr, $mem:expr, $ptr:expr, $size:expr) => {{
        $mem.data_mut(&mut $caller)
            .get_mut($ptr as usize..)
            .and_then(|s| s.get_mut(..$size as usize))
            .expect("failed to read wasm memory")
    }};
}

pub mod network {
    pub mod actor {
        use wasmtime::{ExternRef, Linker};

        use crate::linker::MODULE;

        pub fn set_id(
            linker: &mut Linker<crate::State>,
        ) -> Result<(), crate::error::Error> {
            const NAME: &str = "tq_network_actor_set_id";

            linker.func_wrap2_async::<Option<ExternRef>, i32, ()>(
                MODULE,
                NAME,
                |_caller, actor_ref, id| {
                    Box::new(async move {
                        let actor_ref = actor_ref.expect("actor ref not null");
                        let actor = actor_ref
                            .data()
                            .downcast_ref::<tq_network::ActorHandle>()
                            .expect("actor ref is valid");
                        actor.set_id(id as usize);
                    }) as _
                },
            )?;
            Ok(())
        }

        pub fn shutdown(
            linker: &mut Linker<crate::State>,
        ) -> Result<(), crate::error::Error> {
            const NAME: &str = "tq_network_actor_shutdown";

            linker.func_wrap1_async::<Option<ExternRef>, ()>(
                MODULE,
                NAME,
                |_caller, actor_ref| {
                    Box::new(async move {
                        let actor_ref = actor_ref.expect("actor ref not null");
                        let actor = actor_ref
                            .data()
                            .downcast_ref::<tq_network::ActorHandle>()
                            .expect("actor ref is valid");
                        let _ = actor.shutdown().await;
                    }) as _
                },
            )?;
            Ok(())
        }

        pub fn send(
            linker: &mut Linker<crate::State>,
        ) -> Result<(), crate::error::Error> {
            const NAME: &str = "tq_network_actor_send";
            linker.func_wrap4_async::<Option<ExternRef>, i32, i32, i32, ()>(
                MODULE,
                NAME,
                |mut caller,
                 actor_ref,
                 packet_id,
                 packet_data_ptr,
                 packet_data_len| {
                    Box::new(async move {
                        let actor_ref = actor_ref.expect("actor ref not null");
                        let actor = actor_ref
                            .data()
                            .downcast_ref::<tq_network::ActorHandle>()
                            .expect("actor ref is valid");
                        let mem = memof!(caller);
                        let packet_data = mread!(
                            caller,
                            mem,
                            packet_data_ptr,
                            packet_data_len
                        );
                        let _ =
                            actor.send((packet_id as u16, packet_data)).await;
                    }) as _
                },
            )?;
            Ok(())
        }
    }
}

pub mod rand {
    use wasmtime::Linker;

    use crate::linker::MODULE;
    use rand::Rng;

    pub fn getrandom(
        linker: &mut Linker<crate::State>,
    ) -> Result<(), crate::error::Error> {
        const NAME: &str = "getrandom";
        linker.func_wrap2_async::<i32, i32, i32>(
            MODULE,
            NAME,
            |mut caller, ptr, len| {
                Box::new(async move {
                    let mem = memof!(caller);
                    let slice = mread_mut!(caller, mem, ptr, len);
                    let mut rng = rand::thread_rng();
                    rng.fill(slice);
                    0
                }) as _
            },
        )?;
        Ok(())
    }
}

pub mod log {
    use super::*;
    pub fn trace_event(
        linker: &mut Linker<crate::State>,
    ) -> Result<(), crate::error::Error> {
        const NAME: &str = "trace_event";
        linker
            .func_wrap5_async::<i32, i32, i32, i32, i32, ()>(
                MODULE,
                NAME,
                |mut caller,
                 level,
                 target,
                 target_len,
                 message,
                 message_len| {
                    Box::new(async move {
                        let mem = memof!(caller);
                        let target_slice = mread!(caller, mem, target, target_len);
                        let target = std::str::from_utf8(target_slice).expect("valid utf8");
                        let message_slice = mread!(caller, mem, message, message_len);
                        let message = std::str::from_utf8(message_slice).expect("valid utf8");
                        match level {
                            0 => tracing::error!(target: "runtime", packet = target, %message),
                            1 => tracing::warn!(target: "runtime", packet = target, %message),
                            2 => tracing::info!(target: "runtime", packet = target, %message),
                            3 => tracing::debug!(target: "runtime", packet = target, %message),
                            _ => tracing::trace!(target: "runtime", packet = target, %message),
                        };
                    }) as _
                },
            )?;
        Ok(())
    }
}
