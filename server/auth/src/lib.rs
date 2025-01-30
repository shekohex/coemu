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
    pub msg_account: Module,
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
        const PROCESS_PACKET: &str = "process_packet";
        let packet_len = packet.1.len();

        let mut store = wasmtime::Store::new(&runtime.engine, runtime.state.clone());
        let actor = wasmtime::ExternRef::new(actor.handle());
        match packet.0 {
            msg_connect::MsgConnect::PACKET_ID => {
                let msg_connect = runtime
                    .linker
                    .instantiate_async(&mut store, &runtime.packets.msg_connect)
                    .await?;
                let alloc_packet = msg_connect.get_typed_func::<u32, i32>(&mut store, linker::ALLOC)?;
                let ptr = alloc_packet.call_async(&mut store, packet_len as u32).await?;
                let memory = msg_connect
                    .get_memory(&mut store, linker::MEMORY)
                    .expect("Failed to get memory");
                memory
                    .write(&mut store, ptr as usize, &packet.1)
                    .expect("Failed to write packet to memory");
                let process =
                    msg_connect.get_typed_func::<(i32, i32, Option<ExternRef>), i32>(&mut store, PROCESS_PACKET)?;
                let ret = process
                    .call_async(&mut store, (ptr, packet_len as i32, Some(actor)))
                    .await?;
                match ret {
                    0 => Ok(()),
                    0xdec0de => {
                        tracing::error!("Failed to decode packet: {:#?}", packet);
                        Err(crate::error::Error::InvalidPacket)
                    },
                    0x00f => {
                        tracing::error!("Failed to handle packet: {:#?}", packet);
                        Err(crate::error::Error::InvalidPacket)
                    },
                    code => {
                        tracing::error!("Unknown error: {:#?}", packet);
                        Err(crate::error::Error::Other(format!("Unknown error: {}", code)))
                    },
                }
            },
            msg_account::MsgAccount::PACKET_ID => {
                let msg_connect = runtime
                    .linker
                    .instantiate_async(&mut store, &runtime.packets.msg_account)
                    .await?;
                let alloc_packet = msg_connect.get_typed_func::<u32, i32>(&mut store, linker::ALLOC)?;
                let ptr = alloc_packet.call_async(&mut store, packet_len as u32).await?;
                let memory = msg_connect
                    .get_memory(&mut store, linker::MEMORY)
                    .expect("Failed to get memory");
                memory
                    .write(&mut store, ptr as usize, &packet.1)
                    .expect("Failed to write packet to memory");
                let process =
                    msg_connect.get_typed_func::<(i32, i32, Option<ExternRef>), i32>(&mut store, PROCESS_PACKET)?;
                let ret = process
                    .call_async(&mut store, (ptr, packet_len as i32, Some(actor)))
                    .await?;
                match ret {
                    0 => Ok(()),
                    0xdec0de => {
                        tracing::error!("Failed to decode packet: {:#?}", packet);
                        Err(crate::error::Error::InvalidPacket)
                    },
                    0x00f => {
                        tracing::error!("Failed to handle packet: {:#?}", packet);
                        Err(crate::error::Error::InvalidPacket)
                    },
                    code => {
                        tracing::error!("Unknown error: {:#?}", packet);
                        Err(crate::error::Error::Other(format!("Unknown error: {}", code)))
                    },
                }
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
    linker::log::trace_event(linker)?;
    linker::rand::getrandom(linker)?;

    linker::network::actor::shutdown(linker)?;
    linker::network::actor::send(linker)?;
    linker::network::actor::set_id(linker)?;

    linker::db::account::auth(linker)?;
    linker::db::realm::by_name(linker)?;

    linker::server_bus::check(linker)?;
    linker::server_bus::transfer(linker)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use msg_account::MsgAccount;
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
        let msg_account = Module::from_file(&engine, msg_account::WASM_BINARY.unwrap()).unwrap();

        std::env::set_var("DATABASE_URL", "sqlite::memory:");
        let state = State::init().await.unwrap();

        // Run database migrations
        sqlx::migrate!("../../migrations")
            .run(state.pool())
            .await
            .expect("Failed to migrate database");
        let packets = Packets {
            msg_connect,
            msg_account,
        };

        Runtime {
            state,
            linker,
            engine,
            packets,
        }
    }

    fn setup_logger(verbosity: i32) -> tracing::subscriber::DefaultGuard {
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
        tracing::subscriber::set_default(logger.finish())
    }

    #[tokio::test]
    async fn msg_connect() {
        let _guard = setup_logger(3);
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

    #[tokio::test]
    async fn msg_account() {
        let _guard = setup_logger(3);
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let runtime = create_runtime().await;
        let msg = MsgAccount {
            username: String::from("test").into(),
            password: String::from("test").into(),
            realm: String::from("coemu").into(),
            ..Default::default()
        };
        let actor = Actor::<()>::new(tx);

        let encoded = msg.encode().unwrap();
        Runtime::handle(encoded.clone(), &runtime, &actor).await.unwrap();
        let code = msg_connect_ex::RejectionCode::InvalidPassword;
        let expected_msg = msg_connect_ex::MsgConnectEx::from_code(code);

        let encoded = expected_msg.encode().unwrap();
        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, Message::from(encoded));
    }
}
