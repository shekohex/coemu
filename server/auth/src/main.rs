//! This program encapsulates the account server.
//! The account server is designed to accept login data from the client and
//! to verify that the username and password combination inputted is
//! correct with the database. If the combination is correct, the client
//! will be transferred to the message server of their choice.

use bytes::Bytes;
use msg_account::MsgAccount;
use msg_connect::MsgConnect;
use std::env;
use tq_network::{Actor, PacketDecode, PacketHandler, PacketID, TQCipher};
use tq_server::TQServer;

use auth::{Error, Runtime, State};
use tq_system::ProcessPacket;

struct AuthServer;

impl TQServer for AuthServer {
    type ActorState = ();
    type Cipher = TQCipher;
    type PacketHandler = AuthServerHandler;
}

enum AuthServerHandler {}

#[async_trait::async_trait]
impl PacketHandler for AuthServerHandler {
    type ActorState = ();
    type Error = Error;
    type State = State;

    async fn handle(
        packet: (u16, Bytes),
        state: &Self::State,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        match packet.0 {
            MsgAccount::<Runtime>::PACKET_ID => {
                let packet = MsgAccount::<Runtime>::decode(&packet.1)?;
                packet.process(actor.handle()).await?;
            },
            MsgConnect::<Runtime>::PACKET_ID => {
                let packet = MsgConnect::<Runtime>::decode(&packet.1)?;
                packet.process(actor.handle()).await?;
            },
            _ => {
                tracing::warn!("Unknown packet: {:#?}", packet);
            },
        }
        Ok(())
    }
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
                                      
                                       
Copyright 2020-2023 Shady Khalifa (@shekohex)
     All Rights Reserved.
 "#
    );
    tracing::info!("Starting Auth Server");
    tracing::info!("Initializing State ..");
    let static_state = {
        let state = State::init().await?;
        Box::leak(Box::new(state)) as *mut _
    };
    tracing::info!("Initializing server...");
    let auth_port = env::var("AUTH_PORT")?;
    tracing::info!("Auth Server will be available on {auth_port}");
    // SAFETY: We are the only owner of this Box, and we are deref
    // it. This happens only once, so no one else can access.
    let state = unsafe { &*static_state };
    AuthServer::run(format!("0.0.0.0:{}", auth_port), state).await?;
    unsafe {
        // SAFETY: We are the only owner of this Box, and we are dropping
        // it. This happens at the end of the program, so no one
        // else can access.
        let _ = Box::from_raw(static_state);
    };
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
        .add_directive(format!("tq_db={}", log_level).parse().unwrap())
        .add_directive(format!("tq_serde={}", log_level).parse().unwrap())
        .add_directive(format!("tq_crypto={}", log_level).parse().unwrap())
        .add_directive(format!("tq_codec={}", log_level).parse().unwrap())
        .add_directive(format!("tq_network={}", log_level).parse().unwrap())
        .add_directive(format!("tq_server={}", log_level).parse().unwrap())
        .add_directive(format!("auth={}", log_level).parse().unwrap())
        .add_directive(format!("auth_server={}", log_level).parse().unwrap());
    let logger = tracing_subscriber::fmt()
        .pretty()
        .with_target(true)
        .with_max_level(log_level)
        .with_env_filter(env_filter);
    logger.init();
    Ok(())
}
