use msg_connect_ex::{AccountCredentials, RejectionCode};
use tq_network::IntoErrorPacket;

use crate::types::Realm;

use super::*;

impl<T: Config> MsgTransfer<T> {
    pub fn handle(
        actor: &ActorHandle,
        realm: &str,
    ) -> Result<AccountCredentials, Error> {
        let maybe_realm = T::RealmByName::by_name(realm)?;
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
        if let Err(e) = T::ServerBus::check(&realm) {
            tracing::error!(
                ip = realm.game_ip_address,
                port = realm.game_port,
                realm_id = realm.id,
                error = ?e,
                "Failed to connect to realm"
            );
            T::send(actor, RejectionCode::ServerDown.packet())?;
            T::shutdown(actor)?;
            return Err(e);
        }
        Self::transfer(actor, realm)
    }

    fn transfer(
        actor: &ActorHandle,
        realm: Realm,
    ) -> Result<AccountCredentials, Error> {
        let res = T::ServerBus::transfer(actor, &realm);
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
                    realm_id = realm.id,
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
