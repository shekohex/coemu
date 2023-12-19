use tq_network::ActorHandle;

use crate::types::Realm;

/// Trait for generating login tokens.
pub trait TokenGenerator {
    fn generate_login_token(
        account_id: u32,
        realm_id: u32,
    ) -> Result<u64, crate::Error>;
}

/// Trait for querying realms by name.
#[async_trait::async_trait]
pub trait RealmByName {
    async fn by_name(name: &str) -> Result<Option<Realm>, crate::Error>;
}

#[async_trait::async_trait]
pub trait ServerBus {
    async fn check(realm: &Realm) -> Result<(), crate::Error>;

    async fn transfer(
        actor: &ActorHandle,
        realm: &Realm,
    ) -> Result<u64, crate::Error>;
}
