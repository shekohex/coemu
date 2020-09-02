//! This program encapsulates the account server.
//! The account server is designed to accept login data from the client and
//! to verify that the username and password combination inputted is
//! correct with the database. If the combination is correct, the client
//! will be transferred to the message server of their choice.

use tq_network::{PacketHandler, Server, TQCipher};
use tracing::info;

mod errors;
use errors::Error;

mod state;
use state::State;

mod packets;
use packets::{MsgAccount, MsgConnect};

mod db;

struct AuthServer;

impl Server for AuthServer {
    type ActorState = ();
    type Cipher = TQCipher;
    type PacketHandler = Handler;
}

#[derive(Debug, PacketHandler)]
#[handle(state = ())]
pub enum Handler {
    MsgAccount,
    MsgConnect,
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
    info!("Starting Auth Server");
    info!("Initializing server...");
    let auth_port = dotenv::var("AUTH_PORT")?;
    let ctrlc = tokio::signal::ctrl_c();
    let server = AuthServer::run(format!("0.0.0.0:{}", auth_port));

    info!("Initializing State ..");
    State::init().await?;

    info!("Auth Server will be available on {}", auth_port);

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
