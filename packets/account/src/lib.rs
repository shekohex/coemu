#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

include!(concat!(env!("OUT_DIR"), "/wasm.rs"));

use msg_connect_ex::{MsgConnectEx, RejectionCode};
use msg_transfer::MsgTransfer;
use serde::{Deserialize, Serialize};
use tq_bindings::{host, Resource};
use tq_network::PacketID;
use tq_serde::{String16, TQPassword};

use tq_network::ActorHandle;

#[derive(Default, Debug, Serialize, Deserialize, PacketID)]
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

/// Possible errors that can occur while processing a packet.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// User has entered an invalid username or password.
    #[error("Invalid username or password")]
    InvalidUsernameOrPassword,
    /// Internal Network error.
    #[error(transparent)]
    Network(#[from] tq_network::Error),
    #[error(transparent)]
    Db(#[from] tq_db::Error),
}

#[tq_network::packet_processor(MsgAccount)]
pub fn process(msg: MsgAccount, actor: &Resource<ActorHandle>) -> Result<(), crate::Error> {
    let maybe_accont_id = host::db::account::auth(&msg.username, &msg.password);
    let account_id = match maybe_accont_id {
        Ok(id) => id,
        Err(e) => {
            let res = match e {
                tq_db::Error::AccountNotFound | tq_db::Error::InvalidPassword => {
                    RejectionCode::InvalidPassword.packet()
                },
                _ => {
                    tracing::error!("Error authenticating account: {e}");
                    RejectionCode::TryAgainLater.packet()
                },
            };
            host::network::actor::send(actor, res)?;
            return Ok(());
        },
    };
    host::network::actor::set_id(actor, account_id);
    let res = match MsgTransfer::handle(actor, &msg.realm) {
        Ok(res) => res,
        _ => {
            tracing::warn!(
                %account_id,
                "Failed to transfer account"
            );
            return Ok(());
        },
    };
    let res = MsgConnectEx::forword_connection(res);
    host::network::actor::send(actor, res)?;
    Ok(())
}
