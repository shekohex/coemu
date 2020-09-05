//! This program encapsulates the game server.
//! The game server is designed to accept authenticated data from the
//! account server, load the player's character data, and control the game
//! world environment. Any game structures involving location and the map
//! are processed on this server. Entity intelligence is processed by this
//! server as well.

use async_trait::async_trait;
use tq_network::{Actor, NopCipher, PacketHandler, Server, TQCipher};
use tracing::info;

mod constants;
mod db;
mod entities;
mod systems;
mod utils;
mod world;

mod state;
use state::{ActorState, State};

mod errors;
use errors::Error;

mod packets;
use packets::*;
use std::{env, ops::Deref};

struct GameServer;

#[async_trait]
impl Server for GameServer {
    type ActorState = ActorState;
    type Cipher = TQCipher;
    type PacketHandler = Handler;

    async fn on_disconnected(
        actor: Actor<Self::ActorState>,
    ) -> Result<(), tq_network::Error> {
        tq_network::ActorState::dispose(actor.deref(), &actor)
            .unwrap_or_default();
        Ok(())
    }
}

struct RpcServer;

impl Server for RpcServer {
    type ActorState = ();
    type Cipher = NopCipher;
    type PacketHandler = RpcHandler;
}

#[derive(Copy, Clone, PacketHandler)]
#[handle(state = ActorState)]
pub enum Handler {
    MsgConnect,
    MsgRegister,
    MsgTalk,
    MsgAction,
    MsgItem,
    MsgWalk,
}

#[derive(Copy, Clone, PacketHandler)]
#[handle(state = ())]
pub enum RpcHandler {
    MsgTransfer,
}

#[tokio::main(core_threads = 8)]
async fn main() -> Result<(), Error> {
    dotenv::dotenv()?;
    tracing_subscriber::fmt::init();
    println!(
        r#"
 _____         _____                  
/  __ \       |  ___|                 
| /  \/  ___  | |__  _ __ ___   _   _ 
| |     / _ \ |  __|| '_ ` _ \ | | | |
| \__/\| (_) || |___| | | | | || |_| |
 \____/ \___/ \____/|_| |_| |_| \__,_|
                                      
                                       
Copyright 2020 Shady Khalifa (@shekohex)
     All Rights Reserved.
 "#
    );
    info!("Starting Game Server");
    info!("Initializing server...");

    let game_port = env::var("GAME_PORT")?;
    let rpc_port = env::var("GAME_RPC_PORT")?;

    let ctrlc = tokio::signal::ctrl_c();

    let server = GameServer::run(format!("0.0.0.0:{}", game_port));
    let server = tokio::spawn(server);

    let rpc_server = RpcServer::run(format!("0.0.0.0:{}", rpc_port));
    let rpc_server = tokio::spawn(rpc_server);

    info!("Initializing State ..");
    State::init().await?;

    info!("Game Server will be available on {}", game_port);
    info!("RPC Server will be available on {}", rpc_port);

    tokio::select! {
        _ = ctrlc => {
            info!("Got Ctrl+C Signal!");
        }
        _ = server => {
            info!("Server Is Shutting Down..");
        }
        _ = rpc_server => {
            info!("Rpc Server is Suhtting Down..");
        }
    };
    Ok(())
}
