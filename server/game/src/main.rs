//! This program encapsulates the game server.
//! The game server is designed to accept authenticated data from the
//! account server, load the player's character data, and control the game
//! world environment. Any game structures involving location and the map
//! are processed on this server. Entity intelligence is processed by this
//! server as well.

use async_trait::async_trait;
use std::env;
use tq_network::{Actor, ActorState as _, PacketHandler, TQCipher};
use tq_server::TQServer;

use game::packets::*;
use game::{ActorState, Error, State};

struct GameServer;

#[async_trait]
impl TQServer for GameServer {
    type ActorState = ActorState;
    type Cipher = TQCipher;
    type PacketHandler = Handler;

    /// Get Called right before ending the connection with that client.
    /// good chance to clean up anything related to that actor.
    #[tracing::instrument(skip(state, actor))]
    async fn on_disconnected(
        state: &<Self::PacketHandler as PacketHandler>::State,
        actor: Actor<Self::ActorState>,
    ) -> Result<(), tq_server::Error> {
        if let Ok(entity) = actor.try_entity() {
            let me = entity.as_character().ok_or(Error::CharacterNotFound)?;
            let mymap_id = me.entity().map_id();
            me.save(state).await?;
            me.try_screen()?.remove_from_observers().await?;
            ActorState::dispose(&actor, actor.handle()).await?;
            state.remove_entity(me.id());
            let mymap = state.try_map(mymap_id)?;
            mymap.remove_entity(&entity)?;
        }
        let _ = actor.shutdown().await;
        Ok(())
    }
}

#[derive(Copy, Clone, PacketHandler)]
#[handle(state = State, actor_state = ActorState)]
pub enum Handler {
    MsgConnect,
    MsgRegister,
    MsgTalk,
    MsgAction,
    MsgItem,
    MsgWalk,
    MsgTransfer,
    MsgNpc,
    MsgTaskDialog,
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

    tracing::info!("Initializing State ..");

    let static_state = {
        let state = State::init().await?;
        Box::leak(Box::new(state)) as *mut State
    };

    // SAFETY: We are the only owner of this Box, and we are deref
    // it. This happens only once, so no one else can access.
    let state = unsafe { &*static_state };
    let realm = tq_db::realm::Realm::by_name(state.pool(), "CoEmu")
        .await?
        .ok_or(Error::RealmNotFound)?;
    let game_port = realm.game_port;
    tracing::info!("Game Server will be available on {}", game_port);

    GameServer::run(format!("0.0.0.0:{}", game_port), state).await?;
    unsafe {
        // SAFETY: We are the only owner of this Box, and we are dropping
        // it. This happens at the end of the program, so no one
        // else can access.
        let state = Box::from_raw(static_state);
        state.clean_up().await?;
        // State dropped here.
    };
    tracing::info!("Shutdown.");
    Ok(())
}

fn setup_logger(verbosity: i32) -> Result<(), Error> {
    use tracing::Level;
    use tracing_subscriber::prelude::*;

    let log_level = match verbosity {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let logger = tracing_subscriber::fmt::layer().pretty().with_target(true);
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(format!("tq_db={}", log_level).parse().unwrap())
        .add_directive(format!("tq_serde={}", log_level).parse().unwrap())
        .add_directive(format!("tq_crypto={}", log_level).parse().unwrap())
        .add_directive(format!("tq_codec={}", log_level).parse().unwrap())
        .add_directive(format!("tq_network={}", log_level).parse().unwrap())
        .add_directive(format!("tq_server={}", log_level).parse().unwrap())
        .add_directive(format!("game={}", log_level).parse().unwrap())
        .add_directive(format!("game_server={}", log_level).parse().unwrap());

    #[cfg(feature = "console")]
    let env_filter = env_filter
        .add_directive("tokio=trace".parse().unwrap())
        .add_directive("runtime=trace".parse().unwrap());

    #[cfg(feature = "console")]
    let console_layer = console_subscriber::ConsoleLayer::builder().with_default_env().spawn();

    let registry = tracing_subscriber::registry().with(env_filter).with(logger);

    #[cfg(feature = "console")]
    let registry = registry.with(console_layer);

    registry.init();
    Ok(())
}
