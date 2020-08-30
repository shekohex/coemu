use super::{AccountCredentials, MsgConnectEx, RejectionCode};
use crate::{db, Error};
use network::{
    Actor, NopCipher, PacketDecode, PacketEncode, PacketID, TQCodec,
};
use serde::{Deserialize, Serialize};
use std::io;
use tokio::{net::TcpStream, stream::StreamExt};

/// Defines account parameters to be transferred from the account server to the
/// game server. Account information is supplied from the account database, and
/// used on the game server to transfer authentication and authority level.  
#[derive(Default, Debug, Deserialize, Serialize, PacketID)]
#[packet(id = 1000)]
pub struct MsgTransfer {
    pub account_id: u32,
    #[serde(skip_serializing)]
    pub authentication_token: u32,
    #[serde(skip_serializing)]
    pub authentication_code: u32,
}

impl MsgTransfer {
    pub async fn handle(
        actor: &Actor,
        realm: &str,
    ) -> Result<AccountCredentials, Error> {
        let maybe_realm = db::Realm::by_name(realm).await?;
        // Check if there is a realm with that name
        let realm = match maybe_realm {
            Some(realm) => realm,
            None => {
                actor
                    .send(MsgConnectEx::from_code(RejectionCode::TryAgainLater))
                    .await?;
                actor.shutdown().await?;
                return Err(Error::Other(
                    "Realm Not Found, Closed the socket of the player",
                ));
            },
        };
        // Try to connect to that realm first.
        let stream = TcpStream::connect(format!(
            "{}:{}",
            realm.rpc_ip_address.ip(),
            realm.rpc_port
        ))
        .await;
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                actor
                    .send(MsgConnectEx::from_code(RejectionCode::ServerDown))
                    .await?;
                actor.shutdown().await?;
                return Err(e.into());
            },
        };
        Self::transfer(actor, realm, stream).await
    }

    async fn transfer(
        actor: &Actor,
        realm: db::Realm,
        stream: TcpStream,
    ) -> Result<AccountCredentials, Error> {
        let (mut encoder, mut decoder) =
            TQCodec::new(stream, NopCipher::default()).split();
        let transfer = MsgTransfer {
            account_id: actor.id() as u32,
            ..Default::default()
        };

        let transfer = transfer.encode()?;
        encoder.send(transfer).await?;
        let res = decoder.next().await;
        let res = match res {
            Some(Ok((_, bytes))) => MsgTransfer::decode(&bytes)?,
            Some(Err(e)) => return Err(e.into()),
            None => {
                actor
                    .send(MsgConnectEx::from_code(
                        RejectionCode::ServerTimedOut,
                    ))
                    .await?;
                let io = io::Error::new(
                    io::ErrorKind::ConnectionAborted,
                    "Connection Seems to be closed",
                );
                return Err(io.into());
            },
        };
        Ok(AccountCredentials {
            authentication_code: res.authentication_code,
            authentication_token: res.authentication_token,
            server_ip: realm.game_ip_address.ip().to_string(),
            server_port: realm.game_port as u32,
        })
    }
}
