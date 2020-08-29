use super::AccountCredentials;
use crate::Error;
use network::{NopCipher, PacketDecode, PacketEncode, PacketID, TQCodec};
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
        account_id: u32,
        _realm: &str,
    ) -> Result<AccountCredentials, Error> {
        let stream = TcpStream::connect("192.168.1.4:5817").await?;
        let (mut encoder, mut decoder) =
            TQCodec::new(stream, NopCipher::default()).split();
        let transfer = MsgTransfer {
            account_id,
            ..Default::default()
        };

        let transfer = transfer.encode()?;
        encoder.send(transfer).await?;
        let res = decoder.next().await;
        let res = match res {
            Some(Ok((_, bytes))) => MsgTransfer::decode(&bytes)?,
            Some(Err(e)) => return Err(e.into()),
            None => {
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
            server_ip: "192.168.1.4".into(),
            server_port: 5816,
        })
    }
}
