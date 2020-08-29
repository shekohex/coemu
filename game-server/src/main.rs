use network::{PacketHandler, Server};
use tracing::info;

mod constants;
mod errors;
mod utils;
use errors::Error;

mod packets;
use packets::{MsgAction, MsgConnect, MsgItem, MsgTalk};

#[derive(Server)]
struct GameServer;

#[derive(Copy, Clone, PacketHandler)]
pub enum Handler {
    MsgConnect,
    MsgTalk,
    MsgAction,
    MsgItem,
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
    let ctrlc = tokio::signal::ctrl_c();
    let server = GameServer::run::<Handler>("0.0.0.0:5816");
    info!("Starting Server on 5816");
    tokio::select! {
        _ = ctrlc => {
            info!("Got Ctrl+C Signal!");
        }
        _ = server => {
            info!("Server Is Shutting Down..");
        }
    };
    Ok(())
}
