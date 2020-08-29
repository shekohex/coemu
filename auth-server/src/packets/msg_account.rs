use super::{AccountCredentials, MsgConnectEx};
use crate::{Error, State};
use async_trait::async_trait;
use network::{Actor, PacketID, PacketProcess};
use serde::Deserialize;
use tq_serde::{String16, TQPassword};

#[derive(Debug, Deserialize, PacketID)]
#[packet(id = 1051)]
pub struct MsgAccount {
    pub username: String16,
    pub password: TQPassword,
    pub realm: String16,
    #[serde(skip)]
    pub rejection_code: u32,
    #[serde(skip)]
    pub account_id: i32,
}

#[async_trait]
impl PacketProcess for MsgAccount {
    type Error = Error;

    async fn process(&self, actor: &Actor) -> Result<(), Self::Error> {
        State::global().add_actor(actor);
        let res = MsgConnectEx::forword_connection(AccountCredentials {
            authentication_token: 1001,
            authentication_code: 1002,
            server_ip: "192.168.1.4".into(),
            server_port: 5816,
        });
        actor.send(res).await?;
        Ok(())
    }
}
