//! This program encapsulates the game server.
//! The game server is designed to accept authenticated data from the
//! account server, load the player's character data, and control the game
//! world environment. Any game structures involving location and the map
//! are processed on this server. Entity intelligence is processed by this
//! server as well.

use async_trait::async_trait;
use tq_network::{NopCipher, PacketHandler, Server, TQCipher};

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
use std::env;

struct GameServer;

#[async_trait]
impl Server for GameServer {
    type ActorState = ActorState;
    type Cipher = TQCipher;
    type PacketHandler = Handler;
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

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    let log_verbosity = env::var("LOG_VERBOSITY")
        .map(|s| s.parse::<i32>().unwrap_or(2))
        .unwrap_or(2);
    setup_logger(log_verbosity)?;
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
    tracing::info!("Starting Game Server");
    tracing::info!("Initializing server...");

    let game_port = env::var("GAME_PORT")?;
    let rpc_port = env::var("GAME_RPC_PORT")?;

    let ctrlc = tokio::signal::ctrl_c();

    tracing::info!("Initializing State ..");
    State::init().await?;

    let server = GameServer::run(format!("0.0.0.0:{}", game_port));
    let server = tokio::spawn(server);

    let rpc_server = RpcServer::run(format!("0.0.0.0:{}", rpc_port));
    let rpc_server = tokio::spawn(rpc_server);

    tracing::info!("Game Server will be available on {}", game_port);
    tracing::info!("RPC Server will be available on {}", rpc_port);

    tokio::select! {
        _ = ctrlc => {
            tracing::info!("Got Ctrl+C Signal!");
        }
        _ = server => {
            tracing::info!("Server Is Shutting Down..");
        }
        _ = rpc_server => {
            tracing::info!("Rpc Server is Suhtting Down..");
        }
    };
    State::clean_up().await?;
    tracing::info!("Shutdown.");
    Ok(())
}

fn setup_logger(verbosity: i32) -> Result<(), Error> {
    use tracing::Level;
    let log_level = match verbosity {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(format!("game_server={}", log_level).parse().unwrap());
    let logger = tracing_subscriber::fmt()
        .pretty()
        .with_target(true)
        .with_max_level(log_level)
        .with_env_filter(env_filter);
    logger.init();
    Ok(())
}
