use tq_system::ActorHandle;

use crate::types::Realm;

/// Trait for generating login tokens.
pub trait TokenGenerator {
    fn generate_login_token(
        account_id: u32,
        realm_id: u32,
    ) -> Result<u64, crate::Error>;
}

// A dummy token generator that always returns 0.
impl TokenGenerator for () {
    fn generate_login_token(
        _account_id: u32,
        _realm_id: u32,
    ) -> Result<u64, crate::Error> {
        Ok(0)
    }
}

/// Trait for querying realms by name.
pub trait RealmByName {
    fn by_name(name: &str) -> Result<Option<Realm>, crate::Error>;
}

// A dummy realm query that always returns None.
impl RealmByName for () {
    fn by_name(_name: &str) -> Result<Option<Realm>, crate::Error> { Ok(None) }
}

pub trait ServerBus {
    fn check(realm: &Realm) -> Result<(), crate::Error>;

    fn transfer(
        actor: &ActorHandle,
        realm: &Realm,
    ) -> Result<u64, crate::Error>;
}

// A dummy server bus that always returns an error.
impl ServerBus for () {
    fn check(_realm: &Realm) -> Result<(), crate::Error> {
        Err(crate::Error::RealmUnavailable)
    }

    fn transfer(
        _actor: &ActorHandle,
        _realm: &Realm,
    ) -> Result<u64, crate::Error> {
        Err(crate::Error::RealmUnavailable)
    }
}
