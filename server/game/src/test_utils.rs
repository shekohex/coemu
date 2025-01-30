use futures::future::BoxFuture;
use sqlx::sqlite::SqlitePoolOptions;
use tq_network::Actor;
use tracing_subscriber::prelude::*;

use crate::entities::Character;
use crate::packets::MsgRegister;
use crate::systems::Screen;
use crate::ActorState;

pub async fn with_test_env<'a, F>(log_level: tracing::Level, f: F) -> Result<(), crate::Error>
where
    F: FnOnce(crate::State, [Actor<ActorState>; 2]) -> BoxFuture<'a, Result<(), crate::Error>>,
{
    let root_dir = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to run git command")
        .stdout;
    let root_dir = std::str::from_utf8(&root_dir)?.trim();
    let root_dir = std::path::Path::new(root_dir);
    let data_dir = root_dir.join("data");
    std::env::set_var("DATA_LOCATION", data_dir);

    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(format!("tq_db={}", log_level).parse().unwrap())
        .add_directive(format!("tq_serde={}", log_level).parse().unwrap())
        .add_directive(format!("tq_crypto={}", log_level).parse().unwrap())
        .add_directive(format!("tq_codec={}", log_level).parse().unwrap())
        .add_directive(format!("tq_network={}", log_level).parse().unwrap())
        .add_directive(format!("game={}", log_level).parse().unwrap())
        .add_directive(format!("game_server={}", log_level).parse().unwrap());
    let logger = tracing_subscriber::fmt::layer()
        .pretty()
        .with_target(true)
        .with_test_writer();
    tracing_subscriber::registry().with(env_filter).with(logger).init();

    let pool = SqlitePoolOptions::new()
        .max_connections(42)
        .min_connections(4)
        .connect("sqlite::memory:")
        .await?;
    // Run database migrations
    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");
    let state = crate::State::with_pool(pool).await?;
    let actors = [make_test_actor(&state, 1).await?, make_test_actor(&state, 2).await?];
    f(state, actors).await
}

pub async fn make_test_actor(state: &crate::State, id: usize) -> Result<Actor<ActorState>, crate::Error> {
    let (tx, _rx) = tokio::sync::mpsc::channel(50);
    let actor = Actor::<ActorState>::new(tx);
    actor.set_id(id);
    let inner_character = MsgRegister::build_character_with(
        format!("test{id}"),
        crate::packets::BodyType::MuscularMale,
        crate::packets::BaseClass::Trojan,
        id as _,
        1,
    )?;
    inner_character.save(state.pool()).await?;
    let inner_character = tq_db::character::Character::from_account(state.pool(), id as _)
        .await?
        .expect("Failed to load character");
    let character = Character::new(actor.handle(), inner_character);
    let screen = Screen::new(actor.handle());
    actor.update(character, screen);
    state.insert_entity(actor.entity());
    Ok(actor)
}
