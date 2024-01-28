use super::MsgTalk;
use crate::entities::Character;
use crate::systems::Screen;
use crate::{ActorState, Error, State};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use tq_network::{Actor, IntoErrorPacket, PacketID, PacketProcess};
use tq_serde::{String16, TQMaskedPassword};

#[derive(Debug, Default, Serialize, Deserialize, PacketID)]
#[packet(id = 1001)]
pub struct MsgRegister {
    pub username: String16,
    pub character_name: String16,
    pub password: TQMaskedPassword,
    pub mesh: u16,
    pub class: u16,
    pub token: u32,
}

impl MsgRegister {
    pub fn build_character(&self, account_id: u32, realm_id: u32) -> Result<tq_db::character::Character, Error> {
        Self::build_character_with(
            self.character_name.to_string(),
            BodyType::try_from(self.mesh).map_err(|_| Error::InvalidBodyType)?,
            BaseClass::try_from(self.class).map_err(|_| Error::InvalidClass)?,
            account_id,
            realm_id,
        )
    }

    pub fn build_character_with(
        name: String,
        mesh: BodyType,
        class: BaseClass,
        account_id: u32,
        realm_id: u32,
    ) -> Result<tq_db::character::Character, Error> {
        // Some Math for rand characher.
        let mut rng = rand::rngs::StdRng::from_entropy();

        let avatar = match u16::from(mesh) {
            // For Male
            m if m < 1005 => rng.gen_range(1..49),

            // For Females
            _ => rng.gen_range(201..249),
        };

        let hair_style = rng.gen_range(3..9) * 100 + crate::constants::HAIR_STYLES[rng.gen_range(0..12)];
        let strength = match class {
            BaseClass::Taoist => 2,
            _ => 4,
        };
        let agility = 6;
        let vitality = 12;
        let spirit = match class {
            BaseClass::Taoist => 10,
            _ => 0,
        };

        let health_points = (strength * 3) + (agility * 3) + (spirit * 3) + (vitality * 24);
        let mana_points = spirit * 5;

        let c = tq_db::character::Character {
            account_id: account_id as i32,
            realm_id: realm_id as i32,
            name,
            mesh: u16::from(mesh) as i32,
            avatar,
            hair_style,
            silver: 1000,
            cps: 0,
            current_class: u16::from(class) as i16,
            map_id: 1010,
            x: 61,
            y: 109,
            virtue: 0,
            strength,
            agility,
            vitality,
            spirit,
            health_points,
            mana_points,
            ..Default::default()
        };
        Ok(c)
    }
}

#[derive(Copy, Clone, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum BodyType {
    AgileMale = 1003,
    MuscularMale = 1004,
    AgileFemale = 2001,
    MuscularFemale = 2002,
}

#[derive(Copy, Clone, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum BaseClass {
    Trojan = 10,
    Warrior = 20,
    Archer = 40,
    Taoist = 100,
}

#[async_trait::async_trait]
impl PacketProcess for MsgRegister {
    type ActorState = ActorState;
    type Error = Error;
    type State = State;

    async fn process(&self, state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        let info = state
            .remove_creation_token(self.token)
            .map_err(|_| MsgTalk::register_invalid().error_packet())?;

        if tq_db::character::Character::name_taken(state.pool(), &self.character_name).await? {
            return Err(MsgTalk::register_name_taken().error_packet().into());
        }

        // Validate Data.
        BodyType::try_from(self.mesh).map_err(|_| MsgTalk::register_invalid().error_packet())?;
        BaseClass::try_from(self.class).map_err(|_| MsgTalk::register_invalid().error_packet())?;

        let character_id = self
            .build_character(info.account_id, info.realm_id)?
            .save(state.pool())
            .await?;
        let character = tq_db::character::Character::by_id(state.pool(), character_id).await?;
        let map_id = character.map_id;
        let me = Character::new(actor.handle(), character);
        let screen = Screen::new(actor.handle());
        actor.update(me, screen);
        state.insert_entity(actor.entity());
        // Set player map.
        state
            .try_map(map_id as _)
            .map_err(|_| MsgTalk::register_invalid().error_packet())?
            .insert_entity(actor.entity())
            .await?;

        tracing::info!(
            "Account #{} Created Character #{} with Name {}",
            info.account_id,
            character_id,
            self.character_name
        );
        actor.send(MsgTalk::register_ok()).await?;
        Ok(())
    }
}
