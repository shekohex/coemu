use super::{AccountCredentials, RejectionCode};
use crate::Error;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tq_db::realm::Realm;
use tq_network::{
    Actor, CQCipher, IntoErrorPacket, PacketDecode, PacketEncode, PacketID,
    TQCodec,
};
use tracing::Instrument;

/// Defines account parameters to be transferred from the account server to the
/// game server. Account information is supplied from the account database, and
/// used on the game server to transfer authentication and authority level.  
#[derive(Default, Debug, Deserialize, Serialize, PacketID)]
#[packet(id = 4001)]
pub struct MsgTransfer {
    pub account_id: u32,
    pub realm_id: u32,
    #[serde(skip_serializing)]
    pub token: u64,
}

impl MsgTransfer {
    #[tracing::instrument(skip(state, actor))]
    pub async fn handle(
        state: &crate::State,
        actor: &Actor<()>,
        realm: &str,
    ) -> Result<AccountCredentials, Error> {
        let maybe_realm = Realm::by_name(state.pool(), realm).await?;
        // Check if there is a realm with that name
        let realm = match maybe_realm {
            Some(realm) => realm,
            None => {
                return Err(RejectionCode::TryAgainLater
                    .packet()
                    .error_packet()
                    .into());
            },
        };
        // Try to connect to that realm first.
        let ip = realm.game_ip_address.as_str();
        let port = realm.game_port;
        let stream = TcpStream::connect(format!("{ip}:{port}"))
            .instrument(tracing::info_span!("realm_connect", %ip, %port, realm_id = realm.realm_id))
            .await;
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(
                    %ip,
                    %port,
                    realm_id = realm.realm_id,
                    error = ?e,
                    "Failed to connect to realm"
                );
                actor.send(RejectionCode::ServerDown.packet()).await?;
                actor.shutdown().await?;
                return Err(e.into());
            },
        };
        Self::transfer(actor, realm, stream).await
    }

    #[tracing::instrument(skip(actor, stream), err, fields(realm = realm.name))]
    async fn transfer(
        actor: &Actor<()>,
        realm: Realm,
        stream: TcpStream,
    ) -> Result<AccountCredentials, Error> {
        let cipher = CQCipher::new();
        let (mut encoder, mut decoder) = TQCodec::new(stream, cipher).split();
        let transfer = MsgTransfer {
            account_id: actor.id() as u32,
            realm_id: realm.realm_id as u32,
            ..Default::default()
        };

        let transfer = transfer.encode()?;
        encoder.send(transfer).await?;
        let res = decoder.next().await;
        let res = match res {
            Some(Ok((_, bytes))) => MsgTransfer::decode(&bytes)?,
            Some(Err(e)) => return Err(e.into()),
            None => {
                return Err(RejectionCode::ServerTimedOut
                    .packet()
                    .error_packet()
                    .into());
            },
        };
        Ok(AccountCredentials {
            token: res.token,
            server_ip: realm.game_ip_address,
            server_port: realm.game_port as u32,
        })
    }
}
