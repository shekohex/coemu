#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

// include!(concat!(env!("OUT_DIR"), "/wasm.rs"));

use msg_connect_ex::{AccountCredentials, RejectionCode};
use serde::{Deserialize, Serialize};
use tq_bindings::{host, Resource};
use tq_network::{
    ActorHandle, ErrorPacket, IntoErrorPacket, PacketEncode, PacketID,
};

/// Defines account parameters to be transferred from the account server to the
/// game server. Account information is supplied from the account database, and
/// used on the game server to transfer authentication and authority level.
#[derive(Clone, Default, Debug, Deserialize, Serialize, PacketID)]
#[packet(id = 4001)]
pub struct MsgTransfer {
    account_id: u32,
    realm_id: u32,
    token: u64,
}

impl MsgTransfer {
    pub fn handle(
        actor: &Resource<ActorHandle>,
        realm: &str,
    ) -> Result<AccountCredentials, Error> {
        let maybe_realm = host::realm_by_name(realm)?;
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
        if let Err(e) = host::server_bus_check(&realm) {
            tracing::error!(
                ip = realm.game_ip_address,
                port = realm.game_port,
                realm_id = realm.id,
                error = ?e,
                "Failed to connect to realm"
            );
            host::send(actor, RejectionCode::ServerDown.packet())?;
            host::shutdown(actor);
            return Err(e);
        }
        Self::transfer(actor, realm)
    }

    fn transfer(
        actor: &Resource<ActorHandle>,
        realm: tq_db::realm::Realm,
    ) -> Result<AccountCredentials, Error> {
        let res = host::server_bus_transfer(actor, &realm);
        match res {
            Ok(token) => Ok(AccountCredentials {
                token,
                server_ip: realm.game_ip_address,
                server_port: realm.game_port as u32,
            }),
            Err(e) => {
                tracing::error!(
                    ip = realm.game_ip_address,
                    port = realm.game_port,
                    realm_id = realm.realm_id,
                    error = ?e,
                    "Failed to transfer account"
                );
                Err(RejectionCode::ServerTimedOut
                    .packet()
                    .error_packet()
                    .into())
            },
        }
    }
}

/// Possible errors that can occur while processing a packet.
#[derive(Debug, Clone)]
pub enum Error {
    /// Failed to generate a login token.
    TokenGenerationFailed,
    /// The realm is unavailable.
    RealmUnavailable,
    /// Internal Network error.
    Network(tq_network::Error),
    /// An error packet to be sent to the client.
    Msg(u16, bytes::Bytes),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TokenGenerationFailed => {
                write!(f, "Failed to generate a login token")
            },
            Self::RealmUnavailable => write!(f, "Realm is unavailable"),
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::Msg(id, body) => {
                write!(f, "Error packet: id = {}, body = {:?}", id, body)
            },
        }
    }
}

impl From<tq_network::Error> for Error {
    fn from(e: tq_network::Error) -> Self { Self::Network(e) }
}

impl<T: PacketEncode> From<ErrorPacket<T>> for Error {
    fn from(v: ErrorPacket<T>) -> Self {
        let (id, bytes) = v.0.encode().unwrap();
        Self::Msg(id, bytes)
    }
}

#[tq_network::packet_processor(MsgTransfer)]
fn process(
    msg: MsgTransfer,
    actor: &Resource<ActorHandle>,
) -> Result<(), crate::Error> {
    let token = host::generate_login_token(actor, msg.account_id, msg.realm_id);
    let msg = MsgTransfer {
        account_id: msg.account_id,
        realm_id: msg.realm_id,
        token,
    };
    host::send(&actor, msg)?;
    host::shutdown(&actor);
    Ok(())
}
