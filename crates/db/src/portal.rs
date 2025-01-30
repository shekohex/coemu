#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Portal {
    pub id: i32,
    pub from_map_id: i32,
    pub from_x: i16,
    pub from_y: i16,
    pub to_map_id: i32,
    pub to_x: i16,
    pub to_y: i16,
}

#[cfg(feature = "sqlx")]
impl Portal {
    #[tracing::instrument]
    pub async fn by_map(pool: &sqlx::SqlitePool, from: i32) -> Result<Vec<Self>, crate::Error> {
        use tokio_stream::StreamExt;

        let mut portals = Vec::new();
        let mut s = sqlx::query_as::<_, Self>("SELECT * FROM portals WHERE from_map_id = ?;")
            .bind(from)
            .fetch(pool);
        while let Some(maybe_portal) = s.next().await {
            match maybe_portal {
                Ok(portal) => portals.push(portal),
                Err(error) => {
                    tracing::error!(
                        %error,
                        from_map_id = %from,
                        "Error while loading a portal"
                    );
                },
            }
        }
        Ok(portals)
    }
}
