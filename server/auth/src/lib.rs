//! Auth Server

pub mod error;
pub mod state;

use bytes::Bytes;
pub use state::State;
use tq_network::{Actor, PacketHandler, PacketID};
use wasmtime::{Engine, ExternRef, Linker, Module, Store};

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
        let mut store = Store::new(&runtime.engine, runtime.state.clone());
        let actor = ExternRef::new(actor.handle());
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
            .func_wrap1_async::<Option<ExternRef>, ()>(
                "host",
                "shutdown",
                |caller, actor_ref| {
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
            .unwrap();

        let msg_connect = Module::from_file(
            &engine,
            "../../target/wasm32-unknown-unknown/release/msg_connect.s.wasm",
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

    #[tokio::test]
    async fn test_msg_connect() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let runtime = create_runtime().await;
        let msg = MsgConnect {
            id: 1,
            file_contents: 0,
            file_name: String::from("test").into(),
        };
        let actor = Actor::<()>::new(tx);
        Runtime::handle(
            <MsgConnect as PacketEncode>::encode(&msg).unwrap(),
            &runtime,
            &actor,
        )
        .await
        .unwrap();

        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, Message::Shutdown);
    }
}
