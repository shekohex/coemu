use crate::{Error, State};

/// This struct encapsulates the game character for a player. The player
/// controls the character as the protagonist of the Conquer Online storyline.
/// The character is the persona of the player who controls it. The persona can
/// be altered using different avatars, hairstyles, and body types. The player
/// also controls the character's professions and abilities.
#[derive(Debug, Clone, Default, sqlx::FromRow)]
pub struct Character {
    pub character_id: i32,
    pub account_id: i32,
    pub realm_id: i32,
    pub name: String,
    pub mesh: i32,
    pub avatar: i16,
    pub hair_style: i16,
    pub silver: i64,
    pub cps: i64,
    pub current_class: i16,
    pub previous_class: i16,
    pub rebirths: i16,
    pub level: i16,
    pub experience: i64,
    pub map_id: i32,
    pub x: i16,
    pub y: i16,
    pub virtue: i16,
    pub strength: i16,
    pub agility: i16,
    pub vitality: i16,
    pub spirit: i16,
    pub attribute_points: i16,
    pub health_points: i16,
    pub mana_points: i16,
    pub kill_points: i16,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Location {
    pub map_id: i32,
    pub x: i16,
    pub y: i16,
}

impl Character {
    pub async fn from_account(id: u32) -> Result<Option<Self>, Error> {
        let pool = State::global()?.pool();
        let maybe_character = sqlx::query_as::<_, Self>(
            "SELECT * FROM characters WHERE account_id = ?;",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(maybe_character)
    }

    pub async fn name_taken(name: &str) -> Result<bool, Error> {
        let pool = State::global()?.pool();
        let result = sqlx::query_as::<_, (i32,)>(
            "SELECT EXISTS (SELECT 1 FROM characters WHERE name = ? LIMIT 1);",
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        match result {
            Some((1,)) => Ok(true),
            Some((0,)) => Ok(false),
            // This should never happen.
            _ => Ok(false),
        }
    }

    pub async fn by_id(id: i32) -> Result<Self, Error> {
        let pool = State::global()?.pool();
        let c = sqlx::query_as::<_, Self>(
            "SELECT * FROM characters WHERE character_id = ?;",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;
        Ok(c)
    }

    pub async fn save(self) -> Result<i32, Error> {
        let pool = State::global()?.pool();
        let (id,) = sqlx::query_as::<_, (i32,)>(
            "
            INSERT INTO characters
                (
                    account_id, realm_id, name, mesh, avatar,
                    hair_style, silver, current_class,
                    map_id, x, y, virtue, strength, agility,
                    vitality, spirit, health_points, mana_points
                )
            VALUES 
                (
                    ?, ?, ?, ?, ?, ?,
                    ?, ?, ?, ?, ?, ?,
                    ?, ?, ?, ?, ?, ?
                )
            RETURNING character_id;
            ",
        )
        .bind(self.account_id)
        .bind(self.realm_id)
        .bind(self.name)
        .bind(self.mesh)
        .bind(self.avatar)
        .bind(self.hair_style)
        .bind(self.silver)
        .bind(self.current_class)
        .bind(self.map_id)
        .bind(self.x)
        .bind(self.y)
        .bind(self.virtue)
        .bind(self.strength)
        .bind(self.agility)
        .bind(self.vitality)
        .bind(self.spirit)
        .bind(self.health_points)
        .bind(self.mana_points)
        .fetch_one(pool)
        .await?;
        Ok(id)
    }

    pub async fn update(self) -> Result<(), Error> {
        let pool = State::global()?.pool();
        sqlx::query(
            "
            UPDATE characters
            SET 
                name = ?,
                mesh = ?,
                avatar = ?,
                hair_style = ?,
                silver = ?,
                current_class = ?,
                map_id = ?,
                x = ?, y = ?, 
                virtue = ?,
                strength = ?, 
                agility = ?,
                vitality = ?,
                spirit = ?,
                health_points = ?,
                mana_points = ?
            WHERE character_id = ?;
            ",
        )
        .bind(self.name)
        .bind(self.mesh)
        .bind(self.avatar)
        .bind(self.hair_style)
        .bind(self.silver)
        .bind(self.current_class)
        .bind(self.map_id)
        .bind(self.x)
        .bind(self.y)
        .bind(self.virtue)
        .bind(self.strength)
        .bind(self.agility)
        .bind(self.vitality)
        .bind(self.spirit)
        .bind(self.health_points)
        .bind(self.mana_points)
        .bind(self.character_id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
