mod errors;
use errors::Error;
use network::{PacketHandler, Server};
use state::State;
use tracing::info;

mod packets;
mod state;
use packets::{MsgAccount, MsgConnect};

#[derive(Server)]
struct AuthServer;

#[derive(Debug, PacketHandler)]
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
    State::init();
    let auth_port = dotenv::var("AUTH_PORT")?;
    let ctrlc = tokio::signal::ctrl_c();
    let server =
        AuthServer::run::<Handler, String>(format!("0.0.0.0:{}", auth_port));

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
