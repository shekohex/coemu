use super::MsgTalk;
use crate::{db, systems::Screen, world::Character, ActorState, Error, State};
use async_trait::async_trait;
use num_enum::TryFromPrimitive;
use serde::Deserialize;
use std::convert::TryFrom;
use tq_network::{Actor, IntoErrorPacket, PacketID, PacketProcess};
use tq_serde::{String16, TQPassword};

#[derive(Debug, Default, Deserialize, PacketID)]
#[packet(id = 1001)]
pub struct MsgRegister {
    pub username: String16,
    pub character_name: String16,
    pub password: TQPassword,
    pub mesh: u16,
    pub class: u16,
    pub token: u32,
}

impl MsgRegister {
    pub fn build_character(
        &self,
        account_id: u32,
        realm_id: u32,
    ) -> Result<db::Character, Error> {
        // Some Math for rand characher.

        let avatar = match self.mesh {
            // For Male
            m if m < 1005 => fastrand::i16(1..49),
            // For Females
            _ => fastrand::i16(201..249),
        };

        let hair_style = fastrand::i16(3..9) * 100
            + crate::constants::HAIR_STYLES[fastrand::usize(0..12)];
        let strength = match self.class {
            // Taoist
            100 => 2,
            _ => 4,
        };
        let agility = 6;
        let vitality = 12;
        let spirit = match self.class {
            // Taoist
            100 => 10,
            _ => 0,
        };

        let health_points =
            (strength * 3) + (agility * 3) + (spirit * 3) + (vitality * 24);
        let mana_points = spirit * 5;

        let c = db::Character {
            account_id: account_id as i32,
            realm_id: realm_id as i32,
            name: self.character_name.to_string(),
            mesh: self.mesh as i32,
            avatar,
            hair_style,
            silver: 1000,
            cps: 0,
            current_class: self.class as i16,
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

#[derive(Debug, TryFromPrimitive)]
#[repr(u16)]
pub enum BodyType {
    AgileMale = 1003,
    MuscularMale = 1004,
    AgileFemale = 2001,
    MuscularFemale = 2002,
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u16)]
pub enum BaseClass {
    Trojan = 10,
    Warrior = 20,
    Archer = 40,
    Taoist = 100,
}

#[async_trait]
impl PacketProcess for MsgRegister {
    type ActorState = ActorState;
    type Error = Error;

    async fn process(
        &self,
        actor: &Actor<Self::ActorState>,
    ) -> Result<(), Self::Error> {
        let state = State::global()?;
        let (id, realm_id) = state
            .creation_tokens()
            .remove(&self.token)
            .map(|(_, account_id)| account_id)
            .ok_or_else(|| MsgTalk::register_invalid().error_packet())?;

        if db::Character::name_taken(&self.character_name).await? {
            return Err(MsgTalk::register_name_taken().error_packet().into());
        }

        // Validate Data.
        BodyType::try_from(self.mesh)
            .map_err(|_| MsgTalk::register_invalid().error_packet())?;
        BaseClass::try_from(self.class)
            .map_err(|_| MsgTalk::register_invalid().error_packet())?;

        let character_id = self.build_character(id, realm_id)?.save().await?;
        let character = db::Character::by_id(character_id).await?;
        let map_id = character.map_id;
        let me = Character::new(actor.clone(), character);
        actor.set_character(me.clone()).await?;
        // Set player map.
        state
            .maps()
            .get(&(map_id as u32))
            .ok_or_else(|| MsgTalk::register_invalid().error_packet())?
            .insert_character(me)
            .await?;
        let screen = Screen::new(actor.clone());
        actor.set_screen(screen).await?;

        tracing::info!(
            "Account #{} Created Character #{} with Name {}",
            id,
            character_id,
            self.character_name
        );
        actor.send(MsgTalk::register_ok()).await?;
        Ok(())
    }
}
