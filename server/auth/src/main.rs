//! This program encapsulates the account server.
//! The account server is designed to accept login data from the client and
//! to verify that the username and password combination inputted is
//! correct with the database. If the combination is correct, the client
//! will be transferred to the message server of their choice.

use bytes::Bytes;
use msg_connect::MsgConnect;
use std::env;
use tq_network::{Actor, ActorHandle, PacketDecode, PacketHandler, PacketID, TQCipher};
#[cfg(feature = "server")]
use tq_server::TQServer;
use wasmtime::{Config, Engine, ExternRef, Linker, Module, Store};

use auth::error::Error;
use auth::{Runtime, State};

struct AuthServer;

impl TQServer for AuthServer {
    type ActorState = ();
    type Cipher = TQCipher;
    type PacketHandler = Runtime;
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
    let mut config = Config::new();
    config.async_support(true).wasm_reference_types(true);

    let engine = Engine::new(&config)?;
    let mut linker = Linker::new(&engine);
    auth::add_to_linker(&mut linker)?;
    tracing::info!("Loading Packet and handlers..");

    let msg_connect = Module::from_file(&engine, "./target/wasm32-unknown-unknown/wasm/msg_connect.s.wasm")?;
    tracing::info!("Initializing State ..");
    let state = State::init().await?;
    let packets = auth::Packets { msg_connect };

    let static_runtime = {
        let runtime = Runtime {
            state,
            linker,
            engine,
            packets,
        };
        Box::leak(Box::new(runtime)) as *mut _
    };
    // SAFETY: We are the only owner of this Box, and we are deref
    // it. This happens only once, so no one else can access.
    let runtime: &'static _ = unsafe { &*static_runtime };

    tracing::info!("Starting Auth Server");
    tracing::info!("Initializing server...");
    let auth_port = env::var("AUTH_PORT")?;
    tracing::info!("Auth Server will be available on {auth_port}");
    AuthServer::run(format!("0.0.0.0:{}", auth_port), runtime).await?;
    unsafe {
        // SAFETY: We are the only owner of this Box, and we are dropping
        // it. This happens at the end of the program, so no one
        // else can access.
        let _ = Box::from_raw(static_runtime);
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
