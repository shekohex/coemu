use crate::Error;
use sqlx::SqlitePool;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct Map {
    pub id: i32,
    pub map_id: i32,
    pub path: String,
    pub revive_point_x: i32,
    pub revive_point_y: i32,
    pub flags: i32,
    pub weather: i8,
    pub reborn_map: i32,
    pub color: i32,
}

impl Map {
    /// Loads all maps from the database to add them to the state.
    #[tracing::instrument]
    pub async fn load_all(pool: &SqlitePool) -> Result<Vec<Self>, Error> {
        let mut maps = Vec::new();
        let mut s =
            sqlx::query_as::<_, Self>("SELECT * FROM maps;").fetch(pool);
        while let Some(maybe_map) = s.next().await {
            match maybe_map {
                Ok(map) => maps.push(map),
                Err(error) => {
                    tracing::error!(
                        %error,
                        "Error while loading a map"
                    );
                },
            }
        }
        Ok(maps)
    }

    pub async fn load(
        pool: &SqlitePool,
        id: i32,
    ) -> Result<Option<Self>, Error> {
        let maybe_map =
            sqlx::query_as::<_, Self>("SELECT * FROM maps WHERE id = ?;")
                .bind(id)
                .fetch_optional(pool)
                .await?;
        Ok(maybe_map)
    }
}
