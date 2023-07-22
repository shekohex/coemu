mod error;
mod state;

use std::env;

use error::Error;
use futures::stream::FuturesUnordered;
use game::constants;
use game::packets::{TalkChannel, TalkStyle};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tq_codec::{TQCodec, TQEncoder};
use tq_crypto::{CQCipher, Cipher, NopCipher};
use tq_db::account::Account;
use tq_db::realm::Realm;
use tq_network::{PacketDecode, PacketEncode, PacketID};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    let log_verbosity = env::var("LOG_VERBOSITY")
        .map(|s| s.parse::<i32>().unwrap_or(2))
        .unwrap_or(2);
    setup_logger(log_verbosity)?;
    let state = state::State::init().await?;
    let accounts = create_or_get_accounts(&state).await?;
    let maybe_realm = Realm::by_name(state.pool(), "CoEmu").await?;
    // Check if there is a realm with that name
    let realm = match maybe_realm {
        Some(realm) => realm,
        None => {
            return Err(Error::RealmNotFound);
        },
    };
    let tasks = FuturesUnordered::new();
    for account in accounts {
        let realm = realm.clone();
        let state = state.clone();
        let task = tokio::spawn(async move {
            // Try to connect to that realm's RPC first.
            let ip = realm.rpc_ip_address.as_str();
            let port = realm.rpc_port;
            let stream = TcpStream::connect(format!("{ip}:{port}")).await;
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    return Err(e.into());
                },
            };
            let (mut encoder, mut decoder) =
                TQCodec::new(stream, NopCipher).split();
            let transfer = auth::packets::MsgTransfer {
                account_id: account.account_id as u32,
                realm_id: realm.realm_id as u32,
                ..Default::default()
            };

            let transfer = transfer.encode()?;
            encoder.send(transfer).await?;
            let res = decoder.next().await;
            let res = match res {
                Some(Ok((_, bytes))) => {
                    auth::packets::MsgTransfer::decode(&bytes)?
                },
                Some(Err(e)) => return Err(e.into()),
                None => {
                    return Err(Error::ServerTimedOut);
                },
            };
            // Close the connection and connect to the new realm.
            encoder.close().await?;
            tracing::info!(?account.name, ?realm.name, "Connected to realm");
            let port = realm.game_port;
            let stream =
                TcpStream::connect(format!("127.0.0.1:{port}")).await?;
            let cipher = CQCipher::new();
            let (mut encoder, mut decoder) =
                TQCodec::new(stream, cipher.clone()).split();
            encoder
                .send(
                    game::packets::MsgConnect {
                        token: res.token,
                        build_version: 123,
                        language: String::from("En").into(),
                        file_contents: 10,
                    }
                    .encode()?,
                )
                .await?;
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            cipher.generate_keys(res.token);
            state.tokens().write().insert(account.account_id, res.token);
            while let Some(packet) = decoder.next().await {
                let (id, bytes) = packet?;
                tracing::debug!(?id, "Received packet");
                if id == game::packets::MsgTalk::id() {
                    handle_msg_talk(
                        &state,
                        &account,
                        &mut encoder,
                        game::packets::MsgTalk::decode(&bytes)?,
                    )
                    .await?;
                    continue;
                }
                if id == game::packets::MsgUserInfo::id() {
                    let msg = game::packets::MsgUserInfo::decode(&bytes)?;
                    tracing::debug!(?msg, "Received MsgUserInfo packet");
                    // Move to map 1005 (arena)
                    let msg_cmd = game::packets::MsgTalk {
                        color: 0x00FF_FFFF,
                        channel: TalkChannel::Talk.into(),
                        style: TalkStyle::Normal.into(),
                        character_id: msg.character_id as _,
                        recipient_mesh: 0,
                        sender_mesh: 0,
                        list_count: 4,
                        sender_name: msg.character_name.clone(),
                        recipient_name: constants::ALL_USERS.to_string(),
                        suffix: String::new(),
                        message: String::from("$tele 1005 50 50"),
                    };
                    encoder.send(msg_cmd.encode()?).await?;
                    tokio::time::sleep(std::time::Duration::from_secs(100))
                        .await;
                    continue;
                }
            }
            Ok(())
        });
        tasks.push(task);
    }
    let ctrl_c = tokio::signal::ctrl_c();
    let all_tasks = futures::future::join_all(tasks);
    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Ctrl-C received, shutting down");
        },
        _ = all_tasks => {
            tracing::info!("All tasks finished, shutting down");
        },
    }
    Ok(())
}

async fn create_or_get_accounts(
    state: &state::State,
) -> Result<Vec<Account>, Error> {
    const LIMIT: i64 = 5;
    let mut accounts = Account::all(state.pool(), Some(LIMIT), None).await?;
    tracing::info!(len = %accounts.len(), "Found accounts");
    while accounts.len() < LIMIT as usize {
        let account = Account {
            username: format!("test{}", accounts.len() + 1),
            password: "123456".into(),
            name: Some(format!("Test{}", accounts.len() + 1)),
            ..Default::default()
        }
        .create(state.pool())
        .await?;
        accounts.push(account);
    }
    Ok(accounts)
}

async fn handle_msg_talk(
    state: &state::State,
    account: &Account,
    encoder: &mut TQEncoder<TcpStream, CQCipher>,
    msg: game::packets::MsgTalk,
) -> Result<(), Error> {
    match msg.channel.into() {
        TalkChannel::Login if msg.message.contains("Invalid") => {
            tracing::warn!(?account.name, ?msg, "Invalid password");
            return Err(Error::InvalidPassword);
        },
        TalkChannel::Login if msg.message.eq(constants::ANSWER_OK) => {
            tracing::info!(?account.name, "Logged in");
            return Ok(());
        },
        TalkChannel::Login if msg.message.eq(constants::NEW_ROLE) => {
            // Create a new character
            tracing::info!(?account.name, "Creating character");
            let token = state
                .tokens()
                .read()
                .get(&account.account_id)
                .cloned()
                .ok_or(Error::AccountTokenNotFound)?;
            let msg_register = game::packets::MsgRegister {
                character_name: format!("Test{}", account.account_id).into(),
                class: game::packets::BaseClass::Trojan.into(),
                mesh: game::packets::BodyType::AgileMale.into(),
                token: token as _,
                ..Default::default()
            };
            encoder.send(msg_register.encode()?).await?;
        },
        TalkChannel::Register if msg.message.contains("taken") => {
            tracing::warn!(?account.name, ?msg, "Account already exists");
            return Err(Error::CharacterNameAlreadyTaken);
        },
        TalkChannel::Register if msg.message.eq(constants::ANSWER_OK) => {
            tracing::info!(?account.name, "Account created");
            // Move to map 1005 (arena)
            let msg_cmd = game::packets::MsgTalk {
                color: 0x00FF_FFFF,
                channel: TalkChannel::Talk.into(),
                style: TalkStyle::Normal.into(),
                character_id: 0,
                recipient_mesh: 0,
                sender_mesh: 0,
                list_count: 4,
                sender_name: format!("Test{}", account.account_id),
                recipient_name: constants::ALL_USERS.to_string(),
                suffix: String::new(),
                message: String::from("$tele 1005 50 50"),
            };
            encoder.send(msg_cmd.encode()?).await?;
            return Ok(());
        },
        _ => {
            tracing::debug!(?account.name, ?msg, "Unhandled message");
        },
    }
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
        .add_directive(format!("benchbot={}", log_level).parse().unwrap());
    let logger = tracing_subscriber::fmt()
        .pretty()
        .with_target(true)
        .with_max_level(log_level)
        .with_env_filter(env_filter);
    logger.init();
    Ok(())
}
