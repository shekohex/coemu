mod error;
mod state;

use std::env;

use error::Error;
use pretty_hex::PrettyHex;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tq_codec::TQCodec;
use tq_crypto::{Cipher, NopCipher, TQCipher};
use tq_db::account::Account;
use tq_db::realm::Realm;
use tq_network::{PacketDecode, PacketEncode};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenvy::dotenv()?;
    let log_verbosity = env::var("LOG_VERBOSITY")
        .map(|s| s.parse::<i32>().unwrap_or(2))
        .unwrap_or(2);
    setup_logger(log_verbosity)?;
    let state = state::State::init().await?;
    let account = Account::auth(state.pool(), "shekohex", "123456").await?;
    let maybe_realm = Realm::by_name(state.pool(), "CoEmu").await?;
    // Check if there is a realm with that name
    let realm = match maybe_realm {
        Some(realm) => realm,
        None => {
            return Err(Error::RealmNotFound);
        },
    };
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
    let (mut encoder, mut decoder) = TQCodec::new(stream, NopCipher).split();
    let transfer = auth::packets::MsgTransfer {
        account_id: account.account_id as u32,
        realm_id: realm.realm_id as u32,
        ..Default::default()
    };

    let transfer = transfer.encode()?;
    encoder.send(transfer).await?;
    let res = decoder.next().await;
    let res = match res {
        Some(Ok((_, bytes))) => auth::packets::MsgTransfer::decode(&bytes)?,
        Some(Err(e)) => return Err(e.into()),
        None => {
            return Err(Error::ServerTimedOut);
        },
    };
    // Close the connection and connect to the new realm.
    encoder.close().await?;
    tracing::info!(?realm, "Connected to realm");
    let port = realm.game_port;
    let stream = TcpStream::connect(format!("127.0.0.1:{port}")).await?;
    let cipher = TQCipher::new();
    let (mut encoder, mut decoder) =
        TQCodec::new(stream, cipher.clone()).split();
    encoder
        .send(
            game::packets::MsgConnect {
                token: res.token,
                code: res.code,
                build_version: 123,
                language: String::from("En").into(),
                file_contents: 10,
            }
            .encode()?,
        )
        .await?;
    cipher.generate_keys(res.token, res.code);
    tracing::info!("Sent MsgConnect to realm");
    while let Some(packet) = decoder.next().await {
        let (id, bytes) = packet?;
        let config = pretty_hex::HexConfig {
            title: false,
            ..Default::default()
        };
        let packet_len = bytes.len() + 4;
        tracing::debug!(
            "\nClient -> Server ID({id}) Length({packet_len})\n{:?}",
            bytes.as_ref().hex_conf(config)
        );
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
