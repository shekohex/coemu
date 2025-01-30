/// Realms are configured instances of the game server. This struct defines
/// routing details for authenticated clients to be redirected to. Redirection
/// involves access token leasing, provided by the game server via RPC.
#[derive(Clone, Debug, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Realm {
    pub realm_id: i32,
    pub name: String,
    pub game_ip_address: String,
    pub game_port: i16,
}

#[cfg(feature = "sqlx")]
impl Realm {
    pub async fn by_name(pool: &sqlx::SqlitePool, name: &str) -> Result<Option<Self>, crate::Error> {
        let realm = sqlx::query_as::<_, Self>("SELECT * FROM realms WHERE name = ?;")
            .bind(name)
            .fetch_optional(pool)
            .await?;
        Ok(realm)
    }
}
