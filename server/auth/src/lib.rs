//! Auth Server

pub mod generated {
    wasmtime::component::bindgen!({
        path: "../../packets/connect/wit",
        async: true,
    });
}

pub mod error;
pub mod state;

pub use state::State;
use wasmtime::component::Resource;
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiView};

pub struct Runtime {
    pub state: &'static State,
    pub wasi: WasiCtx,
    pub table: Table,
    pub packets: Packets,
}

pub struct Packets {
    pub msg_connect: generated::MsgConnect,
}

impl WasiView for Runtime {
    fn table(&self) -> &Table { &self.table }

    fn table_mut(&mut self) -> &mut Table { &mut self.table }

    fn ctx(&self) -> &WasiCtx { &self.wasi }

    fn ctx_mut(&mut self) -> &mut WasiCtx { &mut self.wasi }
}

#[async_trait::async_trait]
impl generated::coemu::actor::types::HostActorHandle for Runtime {
    async fn id(
        &mut self,
        r: Resource<generated::coemu::actor::types::ActorHandle>,
    ) -> wasmtime::Result<u32> {
        let actor = self
            .state
            .actor_handles
            .get(&r.rep())
            .ok_or_else(|| error::Error::ActorNotFound)?;
        Ok(actor.id() as u32)
    }

    async fn set_id(
        &mut self,
        r: Resource<generated::coemu::actor::types::ActorHandle>,
        id: u32,
    ) -> wasmtime::Result<()> {
        let actor = self
            .state
            .actor_handles
            .get(&r.rep())
            .ok_or_else(|| error::Error::ActorNotFound)?;
        actor.set_id(id as usize);
        Ok(())
    }

    async fn generate_keys(
        &mut self,
        r: Resource<generated::coemu::actor::types::ActorHandle>,
        seed: u64,
    ) -> wasmtime::Result<()> {
        let actor = self
            .state
            .actor_handles
            .get(&r.rep())
            .ok_or_else(|| error::Error::ActorNotFound)?;
        actor.generate_keys(seed).await?;
        Ok(())
    }

    async fn send(
        &mut self,
        r: Resource<generated::coemu::actor::types::ActorHandle>,
        packet: (u16, Vec<u8>),
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn send_all(
        &mut self,
        r: Resource<generated::coemu::actor::types::ActorHandle>,
        packet: Vec<(u16, Vec<u8>)>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }

    async fn shutdown(
        &mut self,
        r: Resource<generated::coemu::actor::types::ActorHandle>,
    ) -> wasmtime::Result<()> {
        let actor = self
            .state
            .actor_handles
            .get(&r.rep())
            .ok_or_else(|| error::Error::ActorNotFound)?;
        actor.shutdown().await?;
        Ok(())
    }

    fn drop(
        &mut self,
        r: Resource<generated::coemu::actor::types::ActorHandle>,
    ) -> wasmtime::Result<()> {
        // Drop Actor
        Ok(())
    }
}
