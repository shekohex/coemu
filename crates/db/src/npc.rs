#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Npc {
    pub id: i32,
    pub name: String,
    pub kind: i8,
    pub look: i32,
    pub map_id: i32,
    pub x: i16,
    pub y: i16,
    pub base: i8,
    pub sort: i8,
    pub level: i32,
    pub life: i32,
    pub defense: i32,
    pub magic_defense: i32,
}

#[cfg(feature = "sqlx")]
impl Npc {
    #[tracing::instrument]
    pub async fn by_map(pool: &sqlx::SqlitePool, id: i32) -> Result<Vec<Self>, crate::Error> {
        use tokio_stream::StreamExt;

        let mut npcs = Vec::new();
        let mut s = sqlx::query_as::<_, Self>("SELECT * FROM npcs WHERE map_id = ?;")
            .bind(id)
            .fetch(pool);
        while let Some(maybe_npc) = s.next().await {
            match maybe_npc {
                Ok(npc) => npcs.push(npc),
                Err(error) => {
                    tracing::error!(
                        %error,
                        map_id = %id,
                        "Error while loading a npc from the database"
                    );
                },
            }
        }
        Ok(npcs)
    }
}
