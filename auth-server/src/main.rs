use async_trait::async_trait;
mod errors;
use errors::Error;
use handler::Handler;
use network::Server;
use state::State;
use tracing::info;

mod handler;
mod packets;
mod state;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct AuthServer;

#[async_trait]
impl Server for AuthServer {
    type Error = Error;
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
    let ctrlc = tokio::signal::ctrl_c();
    let server = AuthServer::default();
    let server = server.run("0.0.0.0:9958", Handler::default());

    info!("Starting Server on 9958");
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
