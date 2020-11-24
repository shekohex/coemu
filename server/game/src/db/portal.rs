use crate::{Error, State};
use tokio::stream::StreamExt;

#[derive(Debug, Clone, Default)]
pub struct Portal {
    pub id: i32,
    pub from_map_id: i32,
    pub from_x: i16,
    pub from_y: i16,
    pub to_map_id: i32,
    pub to_x: i16,
    pub to_y: i16,
}

impl Portal {
    pub async fn by_map(from: i32) -> Result<Vec<Self>, Error> {
        let pool = State::global()?.pool();
        let mut portals = Vec::new();
        let mut s = sqlx::query_as!(
            Self,
            "SELECT * FROM portals WHERE from_map_id = $1",
            from
        )
        .fetch(pool);
        while let Some(portal) = s.next().await {
            let portal: Self = portal?;
            portals.push(portal);
        }
        Ok(portals)
    }

    pub async fn fix(&self, x: u16, y: u16) -> Result<(), Error> {
        let pool = State::global()?.pool();
        sqlx::query!(
            "UPDATE portals
            SET 
                to_x = $1, to_y = $2
            WHERE
                from_map_id = $3 AND to_map_id = $4",
            x as i16,
            y as i16,
            self.from_map_id,
            self.to_map_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}
