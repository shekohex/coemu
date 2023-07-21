use crate::Error;
use sqlx::SqlitePool;
use tokio_stream::StreamExt;
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct Map {
    pub map_id: i32,
    pub path: String,
    pub revive_point_x: i32,
    pub revive_point_y: i32,
    pub flags: i32,
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
        map_id: i32,
    ) -> Result<Option<Self>, Error> {
        let maybe_map =
            sqlx::query_as::<_, Self>("SELECT * FROM maps WHERE map_id = ?;")
                .bind(map_id)
                .fetch_optional(pool)
                .await?;
        Ok(maybe_map)
    }

    /// Save the map into the database, update it if it is already exists.
    pub async fn save(self, pool: &SqlitePool) -> Result<i32, Error> {
        sqlx::query(
            "
            INSERT INTO maps 
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT (map_id)
            DO UPDATE SET
              path = ?,
              revive_point_x = ?,
              revive_point_y = ?,
              flags = ?;
          ",
        )
        .bind(self.map_id)
        .bind(&self.path)
        .bind(self.revive_point_x)
        .bind(self.revive_point_y)
        .bind(self.flags)
        .bind(&self.path)
        .bind(self.revive_point_x)
        .bind(self.revive_point_y)
        .bind(self.flags)
        .execute(pool)
        .await?;
        Ok(self.map_id)
    }
}
