//! Auth Server

pub mod error;
pub mod linker;
pub mod state;

use bytes::Bytes;
pub use state::State;
use tq_network::{Actor, PacketHandler, PacketID};
use wasmtime::{Engine, ExternRef, Linker, Module};

pub struct Runtime {
    pub state: State,
    pub engine: Engine,
    pub linker: Linker<State>,
    pub packets: Packets,
}

pub struct Packets {
    pub msg_connect: Module,
}

#[async_trait::async_trait]
impl PacketHandler for Runtime {
    type ActorState = ();
    type Error = crate::error::Error;
    type State = Runtime;

    async fn handle(
        packet: (u16, Bytes),
        runtime: &Self::State,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        const ALLOC: &str = "__alloc";
        const PROCESS_PACKET: &str = "process_packet";
        const MEMORY: &str = "memory";
        let mut store = wasmtime::Store::new(&runtime.engine, runtime.state.clone());
        let actor = wasmtime::ExternRef::new(actor.handle());
        match packet.0 {
            msg_connect::MsgConnect::PACKET_ID => {
                let packet_len = packet.1.len();
                let msg_connect = runtime
                    .linker
                    .instantiate_async(&mut store, &runtime.packets.msg_connect)
                    .await?;
                let alloc_packet = msg_connect.get_typed_func::<u32, i32>(&mut store, ALLOC)?;
                let ptr = alloc_packet.call_async(&mut store, packet_len as u32).await?;
                let memory = msg_connect
                    .get_memory(&mut store, MEMORY)
                    .expect("Failed to get memory");
                memory
                    .write(&mut store, ptr as usize, &packet.1)
                    .expect("Failed to write packet to memory");
                let process =
                    msg_connect.get_typed_func::<(i32, i32, Option<ExternRef>), i32>(&mut store, PROCESS_PACKET)?;
                process
                    .call_async(&mut store, (ptr, packet_len as i32, Some(actor)))
                    .await?;
                Ok(())
            },
            _ => {
                tracing::warn!("Unknown packet: {:#?}", packet);
                Ok(())
            },
        }
    }
}

/// Add the runtime to the linker.
pub fn add_to_linker(linker: &mut Linker<crate::State>) -> Result<(), error::Error> {
    linker::network::actor::shutdown(linker)?;
    linker::network::actor::send(linker)?;
    linker::log::trace_event(linker)?;
    linker::rand::getrandom(linker)?;
    linker::db::account::auth(linker)?;
    linker::db::realm::by_name(linker)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use msg_connect::MsgConnect;
    use tq_network::{Message, PacketEncode};
    use wasmtime::Config;

    use super::*;

    async fn create_runtime() -> Runtime {
        let mut config = Config::new();
        config
            .async_support(true)
            .wasm_reference_types(true)
            .wasm_backtrace(true)
            .wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable)
            .native_unwind_info(true)
            .coredump_on_trap(true);

        let engine = Engine::new(&config).unwrap();
        let mut linker = Linker::new(&engine);
        add_to_linker(&mut linker).unwrap();
        let msg_connect = Module::from_file(&engine, msg_connect::WASM_BINARY.unwrap()).unwrap();
        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        let state = State::init().await.unwrap();
        let packets = Packets { msg_connect };

        Runtime {
            state,
            linker,
            engine,
            packets,
        }
    }

    fn setup_logger(verbosity: i32) {
        use tracing::Level;
        let log_level = match verbosity {
            0 => Level::ERROR,
            1 => Level::WARN,
            2 => Level::INFO,
            3 => Level::DEBUG,
            _ => Level::TRACE,
        };

        let env_filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(format!("tq_db={}", log_level).parse().unwrap())
            .add_directive(format!("tq_serde={}", log_level).parse().unwrap())
            .add_directive(format!("tq_crypto={}", log_level).parse().unwrap())
            .add_directive(format!("tq_codec={}", log_level).parse().unwrap())
            .add_directive(format!("tq_network={}", log_level).parse().unwrap())
            .add_directive(format!("tq_server={}", log_level).parse().unwrap())
            .add_directive(format!("runtime={}", log_level).parse().unwrap())
            .add_directive(format!("auth={}", log_level).parse().unwrap());
        let logger = tracing_subscriber::fmt()
            .pretty()
            .with_test_writer()
            .with_target(true)
            .with_max_level(log_level)
            .with_env_filter(env_filter);
        logger.init();
    }

    #[tokio::test]
    async fn test_msg_connect() {
        setup_logger(3);
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let runtime = create_runtime().await;
        let msg = MsgConnect {
            id: 1,
            file_contents: 0,
            file_name: String::from("test").into(),
        };
        let actor = Actor::<()>::new(tx);

        let encoded = <MsgConnect as PacketEncode>::encode(&msg).unwrap();
        Runtime::handle(encoded.clone(), &runtime, &actor).await.unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, Message::Shutdown);
    }
}
