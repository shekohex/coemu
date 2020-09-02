use crate::{Error, State};
use tokio::stream::StreamExt;
#[derive(Debug, Clone, Default)]
pub struct Map {
    pub map_id: i32,
    pub path: String,
    pub revive_point_x: i32,
    pub revive_point_y: i32,
    pub flags: i32,
}

impl Map {
    /// Loads all maps from the database to add them to the state.
    pub async fn load_all() -> Result<Vec<Self>, Error> {
        let pool = State::global()?.pool();
        let mut maps = Vec::new();
        let mut s = sqlx::query_as!(Self, "SELECT * FROM maps").fetch(pool);
        while let Some(map) = s.next().await {
            let map = map?;
            maps.push(map);
        }
        Ok(maps)
    }

    pub async fn load(map_id: i32) -> Result<Option<Self>, Error> {
        let pool = State::global()?.pool();
        let maybe_map = sqlx::query_as!(
            Self,
            "SELECT * FROM maps WHERE map_id = $1",
            map_id
        )
        .fetch_optional(pool)
        .await?;
        Ok(maybe_map)
    }

    /// Save the map into the database, update it if it is already exists.
    pub async fn save(self) -> Result<i32, Error> {
        let pool = State::global()?.pool();
        let res = sqlx::query!(
            "
            INSERT INTO maps 
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (map_id)
            DO UPDATE SET 
              path = $2,
              revive_point_x = $3,
              revive_point_y = $4,
              flags = $5
            RETURNING map_id
          ",
            self.map_id,
            self.path,
            self.revive_point_x,
            self.revive_point_y,
            self.flags
        )
        .fetch_one(pool)
        .await?;
        Ok(res.map_id)
    }
}
