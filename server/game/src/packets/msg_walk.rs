use crate::constants::{WALK_XCOORDS, WALK_YCOORDS};
use crate::state::State;
use crate::systems::TileType;
use crate::{ActorState, Error};
use async_trait::async_trait;
use num_enum::FromPrimitive;
use primitives::Location;
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};

use super::{MsgTalk, TalkChannel};

#[derive(Debug, FromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum MovementType {
    Walk = 0,
    Run = 1,
    Shift = 2,
    #[num_enum(default)]
    Unknwon,
}

/// This packet encapsulates a character's ground movement on a map. The
/// movement packet specifies the type of movement being performed
/// and the direction the player as it moves on the map. The packet shows
/// movements from actors on the server, and should be sent back to the actor to
/// complete the movement.
#[derive(Debug, Serialize, Deserialize, Clone, PacketID)]
#[packet(id = 1005)]
pub struct MsgWalk {
    character_id: u32,
    direction: u8,
    movement_type: u8,
}

#[async_trait]
impl PacketProcess for MsgWalk {
    type ActorState = ActorState;
    type Error = Error;
    type State = State;

    /// processes a character movement for the actor. It checks if
    /// the movement is valid, then distributes it to observing players. if
    /// the movement is invalid, the packet will not be sent back and the actor
    /// will be teleported back to the character's original position.
    async fn process(&self, state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        let direction = (self.direction % 8) as usize;
        let entity = actor.entity();
        let me = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let current_location = me.entity().location();
        let offset = ((WALK_XCOORDS[direction] as u16), (WALK_YCOORDS[direction] as u16));
        let x = current_location.x.wrapping_add(offset.0);
        let y = current_location.y.wrapping_add(offset.1);
        let map = state.try_map(me.entity().map_id())?;
        match map.tile(x, y) {
            Some(tile) if tile.access > TileType::Npc => {
                // The packet is valid. Assign character data:
                // Send the movement back to the message server and client:
                me.entity().set_location(Location::new(x, y, direction as _));
                me.set_elevation(tile.elevation);
                actor.send(self.clone()).await?;
                map.update_region_for(actor.entity());
                let myscreen = actor.screen();
                myscreen.send_movement(state, self.clone()).await?;
            },
            Some(_) | None => {
                let msg = MsgTalk::from_system(me.id(), TalkChannel::TopLeft, "Invalid Location");
                actor.send(msg).await?;
                me.kick_back().await?;
                return Ok(());
            },
        };
        Ok(())
    }
}
