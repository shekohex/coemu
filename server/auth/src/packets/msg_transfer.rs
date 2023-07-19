use super::{AccountCredentials, RejectionCode};
use crate::{db, Error};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tq_network::{
    Actor, IntoErrorPacket, NopCipher, PacketDecode, PacketEncode, PacketID,
    TQCodec,
};

/// Defines account parameters to be transferred from the account server to the
/// game server. Account information is supplied from the account database, and
/// used on the game server to transfer authentication and authority level.  
#[derive(Default, Debug, Deserialize, Serialize, PacketID)]
#[packet(id = 4001)]
pub struct MsgTransfer {
    pub account_id: u32,
    pub realm_id: u32,
    #[serde(skip_serializing)]
    pub token: u32,
    #[serde(skip_serializing)]
    pub code: u32,
}

impl MsgTransfer {
    pub async fn handle(
        actor: &Actor<()>,
        realm: &str,
    ) -> Result<AccountCredentials, Error> {
        let maybe_realm = db::Realm::by_name(realm).await?;
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
        let ip = realm.rpc_ip_address.as_str();
        let port = realm.rpc_port;
        let stream = TcpStream::connect(format!("{ip}:{port}")).await;
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                actor.send(RejectionCode::ServerDown.packet()).await?;
                actor.shutdown().await?;
                return Err(e.into());
            },
        };
        Self::transfer(actor, realm, stream).await
    }

    async fn transfer(
        actor: &Actor<()>,
        realm: db::Realm,
        stream: TcpStream,
    ) -> Result<AccountCredentials, Error> {
        let (mut encoder, mut decoder) =
            TQCodec::new(stream, NopCipher).split();
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
            code: res.code,
            server_ip: realm.game_ip_address,
            server_port: realm.game_port as u32,
        })
    }
}
