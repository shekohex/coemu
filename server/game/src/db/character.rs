use crate::{Error, State};
use chrono::{DateTime, Utc};

/// This struct encapsulates the game character for a player. The player
/// controls the character as the protagonist of the Conquer Online storyline.
/// The character is the persona of the player who controls it. The persona can
/// be altered using different avatars, hairstyles, and body types. The player
/// also controls the character's professions and abilities.
#[derive(Debug, Clone)]
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
    pub created_at: DateTime<Utc>,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            character_id: Default::default(),
            account_id: Default::default(),
            realm_id: Default::default(),
            name: Default::default(),
            mesh: Default::default(),
            avatar: Default::default(),
            hair_style: Default::default(),
            silver: Default::default(),
            cps: Default::default(),
            current_class: Default::default(),
            previous_class: Default::default(),
            rebirths: Default::default(),
            level: Default::default(),
            experience: Default::default(),
            map_id: Default::default(),
            x: Default::default(),
            y: Default::default(),
            virtue: Default::default(),
            strength: Default::default(),
            agility: Default::default(),
            vitality: Default::default(),
            spirit: Default::default(),
            attribute_points: Default::default(),
            health_points: Default::default(),
            mana_points: Default::default(),
            kill_points: Default::default(),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug)]
pub struct Location {
    pub map_id: i32,
    pub x: i16,
    pub y: i16,
}

impl Character {
    pub async fn from_account(id: u32) -> Result<Option<Self>, Error> {
        let pool = State::global()?.pool();
        let maybe_character = sqlx::query_as!(
            Self,
            "SELECT * FROM characters WHERE account_id = $1",
            id as i32
        )
        .fetch_optional(pool)
        .await?;
        Ok(maybe_character)
    }

    pub async fn name_taken(name: &str) -> Result<bool, Error> {
        let pool = State::global()?.pool();
        let taken = sqlx::query!(
            "SELECT EXISTS (SELECT character_id FROM characters WHERE name = $1)",
            name
        )
        .fetch_one(pool)
        .await?
        .exists
        .unwrap_or(true);
        Ok(taken)
    }

    pub async fn by_id(id: i32) -> Result<Self, Error> {
        let pool = State::global()?.pool();
        let c = sqlx::query_as!(
            Self,
            "SELECT * FROM characters WHERE character_id = $1",
            id
        )
        .fetch_one(pool)
        .await?;
        Ok(c)
    }

    pub async fn save(self) -> Result<i32, Error> {
        let pool = State::global()?.pool();
        let result = sqlx::query!(
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
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10, $11, $12,
                    $13, $14, $15, $16, $17, $18
                )
            RETURNING character_id
            ",
            self.account_id,
            self.realm_id,
            self.name,
            self.mesh,
            self.avatar,
            self.hair_style,
            self.silver,
            self.current_class,
            self.map_id,
            self.x,
            self.y,
            self.virtue,
            self.strength,
            self.agility,
            self.vitality,
            self.spirit,
            self.health_points,
            self.mana_points,
        )
        .fetch_one(pool)
        .await?;
        Ok(result.character_id)
    }
}
