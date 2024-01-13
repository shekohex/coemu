//! Auth Server

pub mod error;
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
        const ALLOC_PACKET: &str = "alloc_packet";
        const PROCESS_PACKET: &str = "process_packet";
        const MEMORY: &str = "memory";
        let mut store =
            wasmtime::Store::new(&runtime.engine, runtime.state.clone());
        let actor = wasmtime::ExternRef::new(actor.handle());
        match packet.0 {
            msg_connect::MsgConnect::PACKET_ID => {
                let packet_len = packet.1.len();
                let msg_connect = runtime
                    .linker
                    .instantiate_async(&mut store, &runtime.packets.msg_connect)
                    .await?;
                let alloc_packet = msg_connect
                    .get_typed_func::<u32, i32>(&mut store, ALLOC_PACKET)?;
                let ptr = alloc_packet
                    .call_async(&mut store, packet_len as u32)
                    .await?;
                let memory = msg_connect
                    .get_memory(&mut store, MEMORY)
                    .expect("Failed to get memory");
                memory
                    .write(&mut store, ptr as usize, &packet.1)
                    .expect("Failed to write packet to memory");
                let process = msg_connect
                    .get_typed_func::<(i32, i32, Option<ExternRef>), i32>(
                        &mut store,
                        PROCESS_PACKET,
                    )?;
                process
                    .call_async(
                        &mut store,
                        (ptr, packet_len as i32, Some(actor)),
                    )
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

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use msg_connect::MsgConnect;
    use tq_network::{ActorHandle, Message, PacketEncode};
    use wasmtime::Config;

    use super::*;

    async fn create_runtime() -> Runtime {
        let mut config = Config::new();
        config.async_support(true);
        config.wasm_reference_types(true);

        let engine = Engine::new(&config).unwrap();
        let mut linker = Linker::new(&engine);
        linker
            .func_wrap5_async::<i32, i32, i32, i32, i32, ()>(
                "host",
                "trace_event",
                |mut caller,
                 level,
                 target,
                 target_len,
                 message,
                 message_len| {
                    Box::new(async move {
                        let memory = caller
                            .get_export("memory")
                            .and_then(|e| e.into_memory())
                            .expect("Failed to get memory");
                        let mut target_buf = vec![0; target_len as usize];
                        let mut message_buf = vec![0; message_len as usize];
                        memory
                            .read(&caller, target as usize, &mut target_buf)
                            .expect("Failed to read target from memory");
                        memory
                            .read(&caller, message as usize, &mut message_buf)
                            .expect("Failed to read message from memory");
                        let target = String::from_utf8(target_buf).unwrap();
                        let message = String::from_utf8(message_buf).unwrap();
                        match level {
                            0 => tracing::error!(%target, %message),
                            1 => tracing::warn!(%target, %message),
                            2 => tracing::info!(%target, %message),
                            3 => tracing::debug!(%target, %message),
                            _ => tracing::trace!(%target, %message),
                        };
                    }) as _
                },
            )
            .unwrap()
            .func_wrap1_async::<Option<ExternRef>, ()>(
                "host",
                "shutdown",
                |_caller, actor_ref| {
                    Box::new(async move {
                        let actor_ref = actor_ref.unwrap();
                        let actor = actor_ref
                            .data()
                            .downcast_ref::<ActorHandle>()
                            .unwrap();
                        let _ = actor.shutdown().await;
                    }) as _
                },
            )
            .unwrap()
            .func_wrap4_async::<Option<ExternRef>, i32, i32, i32, ()>(
                "host",
                "send",
                |mut caller,
                 actor_ref,
                 packet_id,
                 packet_data_ptr,
                 packet_data_len| {
                    Box::new(async move {
                        let actor_ref = actor_ref.unwrap();
                        let actor = actor_ref
                            .data()
                            .downcast_ref::<ActorHandle>()
                            .unwrap();
                        let memory = caller
                            .get_export("memory")
                            .and_then(|e| e.into_memory())
                            .expect("Failed to get memory");
                        let mut packet_data =
                            BytesMut::with_capacity(packet_data_len as usize);
                        packet_data.resize(packet_data_len as usize, 0);
                        memory
                            .read(
                                caller,
                                packet_data_ptr as usize,
                                &mut packet_data,
                            )
                            .expect("Failed to read packet from memory");
                        let _ = actor
                            .send((packet_id as u16, packet_data.freeze()))
                            .await;
                    }) as _
                },
            )
            .unwrap();

        let msg_connect = Module::from_file(
            &engine,
            "../../target/wasm32-unknown-unknown/wasm/msg_connect.s.wasm",
        )
        .unwrap();
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
        Runtime::handle(encoded.clone(), &runtime, &actor)
            .await
            .unwrap();

        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, Message::Packet(encoded.0, encoded.1));

        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, Message::Shutdown);
    }
}
