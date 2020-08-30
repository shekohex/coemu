use crate::{Error, State};
use chrono::{DateTime, Utc};
use sqlx::types::ipnetwork::IpNetwork;

#[derive(Debug)]
pub struct Realm {
    pub realm_id: i32,
    pub name: String,
    pub game_ip_address: IpNetwork,
    pub game_port: i16,
    pub rpc_ip_address: IpNetwork,
    pub rpc_port: i16,
    pub created_at: DateTime<Utc>,
}

impl Realm {
    pub async fn by_name(name: &str) -> Result<Option<Self>, Error> {
        let pool = State::global()?.pool();
        let realm =
            sqlx::query_as!(Self, "SELECT * FROM realms WHERE name = $1", name)
                .fetch_optional(pool)
                .await?;
        Ok(realm)
    }
}
