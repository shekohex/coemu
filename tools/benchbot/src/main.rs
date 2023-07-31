mod error;
mod state;

use core::time::Duration;
use std::env;

use error::Error;
use futures::stream::FuturesUnordered;
use game::packets::{self, ActionType, TalkChannel, TalkStyle};
use game::utils::LoHi;
use game::{constants, utils};
use rand::{Rng, SeedableRng};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tq_codec::{TQCodec, TQEncoder};
use tq_crypto::{CQCipher, Cipher};
use tq_db::account::Account;
use tq_db::realm::Realm;
use tq_network::{PacketDecode, PacketEncode, PacketID};

const NUM_OF_BOTS: i64 = 1200;
const MAX_ACTION_DELAY: Duration = Duration::from_millis(300);

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
    let local_ip = local_ip_address::local_ip().expect("local ip");
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
            // let ip = realm.game_ip_address.as_str();
            let port = realm.game_port;
            let stream = TcpStream::connect(format!("{local_ip}:{port}")).await;
            let stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    return Err(e.into());
                },
            };
            let cipher = CQCipher::new();
            let (mut encoder, mut decoder) =
                TQCodec::new(stream, cipher).split();
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
                    packets::MsgConnect {
                        token: res.token,
                        build_version: 123,
                        language: String::from("En").into(),
                        file_contents: 10,
                    }
                    .encode()?,
                )
                .await?;
            state.tokens().write().insert(account.account_id, res.token);
            cipher.generate_keys(res.token);
            while let Some(packet) = decoder.next().await {
                let (id, bytes) = packet?;
                match id {
                    packets::MsgTalk::PACKET_ID => {
                        handle_msg_talk(
                            &state,
                            &account,
                            &mut encoder,
                            packets::MsgTalk::decode(&bytes)?,
                        )
                        .await?;
                    },
                    packets::MsgAction::PACKET_ID => {
                        let msg = packets::MsgAction::decode(&bytes)?;
                        do_random_stuff(&state, &account, &mut encoder, msg)
                            .await?;
                    },
                    packets::MsgUserInfo::PACKET_ID => {
                        let msg = packets::MsgUserInfo::decode(&bytes)?;
                        tracing::trace!(?msg, "Received MsgUserInfo packet");
                        let msg_leave_booth = packets::MsgAction {
                            client_timestamp: utils::current_ts(),
                            character_id: msg.character_id as _,
                            data1: 0,
                            data2: 0,
                            details: 0,
                            action_type: ActionType::LeaveBooth.into(),
                        };
                        encoder.send(msg_leave_booth.encode()?).await?;
                        let msg_setlocation = packets::MsgAction {
                            client_timestamp: utils::current_ts(),
                            character_id: msg.character_id as _,
                            data1: 0,
                            data2: 0,
                            details: 0,
                            action_type: ActionType::SendLocation.into(),
                        };
                        encoder.send(msg_setlocation.encode()?).await?;
                        do_random_stuff(
                            &state,
                            &account,
                            &mut encoder,
                            msg_setlocation,
                        )
                        .await?;
                    },
                    packets::MsgPlayer::PACKET_ID => {
                        let msg = packets::MsgPlayer::decode(&bytes)?;
                        tracing::trace!(%msg.character_name, "We now see other players");
                    },
                    _ => {
                        tracing::trace!(%id, "Received unknown packet");
                    },
                }
            }
            tracing::debug!(?account.name, ?realm.name, "Disconnected");
            encoder.close().await?;
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
    let mut accounts =
        Account::all(state.pool(), Some(NUM_OF_BOTS), Some(2)).await?;
    while accounts.len() < NUM_OF_BOTS as usize {
        let account = Account {
            username: format!("bot{}", accounts.len() + 1),
            password: "123456".into(),
            name: Some(format!("Bot{}", accounts.len() + 1)),
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
    msg: packets::MsgTalk,
) -> Result<(), Error> {
    tracing::trace!(?msg, "Received MsgTalk packet");
    match msg.channel.into() {
        TalkChannel::Login if msg.message.contains("Invalid") => {
            tracing::warn!(?account.name, ?msg, "Invalid password");
            return Err(Error::InvalidPassword);
        },
        TalkChannel::Login if msg.message.eq(constants::ANSWER_OK) => {
            tracing::info!(?account.name, "Logged in");
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
            let msg_register = packets::MsgRegister {
                character_name: format!("bot{}", account.account_id).into(),
                class: packets::BaseClass::Trojan.into(),
                mesh: packets::BodyType::AgileMale.into(),
                token: token as _,
                ..Default::default()
            };
            tracing::debug!(?msg_register, "Sending MsgRegister packet");
            encoder.send(msg_register.encode()?).await?;
        },
        TalkChannel::Register if msg.message.contains("taken") => {
            tracing::warn!(?account.name, ?msg, "Account already exists");
            return Err(Error::CharacterNameAlreadyTaken);
        },
        TalkChannel::Register if msg.message.eq(constants::ANSWER_OK) => {
            tracing::info!(?account.name, "Account created");
        },
        _ => {
            tracing::trace!(?account.name, ?msg, "Unhandled message");
        },
    }
    Ok(())
}

async fn do_random_stuff(
    _state: &state::State,
    account: &Account,
    encoder: &mut TQEncoder<TcpStream, CQCipher>,
    msg: packets::MsgAction,
) -> Result<(), Error> {
    let (w, h) = (70, 70);
    let target_map_id = 1005;
    match msg.action_type.into() {
        ActionType::SendLocation => {
            let mut rng =
                rand::rngs::StdRng::seed_from_u64(msg.character_id as _);
            // Move to map 1005 (arena) if we are not there already
            let map_id = msg.data1;
            if map_id == target_map_id {
                return Ok(());
            }
            let x = rng.gen_range(35..60);
            let y = rng.gen_range(35..60);
            let msg_cmd = packets::MsgTalk {
                color: 0x00FF_FFFF,
                channel: TalkChannel::Talk.into(),
                style: TalkStyle::Normal.into(),
                character_id: msg.character_id as _,
                recipient_mesh: 0,
                sender_mesh: 0,
                list_count: 4,
                sender_name: String::new(),
                recipient_name: constants::ALL_USERS.to_string(),
                suffix: String::new(),
                message: format!("$tele {target_map_id} {x} {y}"),
            };
            encoder.send(msg_cmd.encode()?).await?;
        },
        ActionType::Teleport => {
            // Maybe that was an invalid move, try again
            let (my_x, my_y) = (msg.data2.lo(), msg.data2.hi());
            // random x, y but not too far
            let (x, y) = loop {
                let mut rng = rand::thread_rng();
                let x_inc = rng.gen_range(-18..18);
                let y_inc = rng.gen_range(-18..18);
                let x = (my_x as i16 + x_inc) as u16;
                let y = (my_y as i16 + y_inc) as u16;
                let in_range = |x, y| x > w / 2 && y > h / 2 && x < w && y < h;
                if in_range(x, y) {
                    break (x, y);
                }
            };
            let mut jump = msg.clone();
            jump.action_type = ActionType::Jump.into();
            jump.data1 = u32::constract(y, x);
            // Simulate a delay before sending the packet
            tokio::time::sleep(MAX_ACTION_DELAY).await;
            encoder.send(jump.encode()?).await?;
        },
        ActionType::Jump => {
            // that was a valid move, now jump again.
            let (my_x, my_y) = (msg.data1.lo(), msg.data1.hi());
            // random x, y but not too far
            let (x, y) = loop {
                let mut rng = rand::thread_rng();
                let x_inc = rng.gen_range(-18..18);
                let y_inc = rng.gen_range(-18..18);
                let x = (my_x as i16 + x_inc) as u16;
                let y = (my_y as i16 + y_inc) as u16;
                let in_range = |x, y| x > w / 2 && y > h / 2 && x < w && y < h;
                if in_range(x, y) {
                    break (x, y);
                }
            };
            let mut jump = msg.clone();
            jump.data1 = u32::constract(y, x);
            tokio::time::sleep(MAX_ACTION_DELAY).await;
            encoder.send(jump.encode()?).await?;
        },
        others => {
            tracing::debug!(
                ?account.name,
                ty = ?others,
                ?msg,
                "Unhandled action"
            );
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
