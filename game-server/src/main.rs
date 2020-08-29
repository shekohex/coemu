use network::{NopCipher, PacketHandler, Server};
use tracing::info;

mod constants;
mod errors;
mod utils;
use errors::Error;

mod packets;
use packets::{MsgAction, MsgConnect, MsgItem, MsgTalk, MsgTransfer};

#[derive(Server)]
struct GameServer;

#[derive(Copy, Clone, PacketHandler)]
pub enum Handler {
    MsgConnect,
    MsgTalk,
    MsgAction,
    MsgItem,
}

#[derive(Copy, Clone, PacketHandler)]
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

    let game_port = dotenv::var("GAME_PORT")?;
    let rpc_port = dotenv::var("GAME_RPC_PORT")?;

    let ctrlc = tokio::signal::ctrl_c();

    let server =
        GameServer::run::<Handler, String>(format!("0.0.0.0:{}", game_port));
    let server = tokio::spawn(server);

    let rpc_server = GameServer::run_with_cipher::<RpcHandler, String, NopCipher>(
        format!("0.0.0.0:{}", rpc_port),
    );
    let rpc_server = tokio::spawn(rpc_server);

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
