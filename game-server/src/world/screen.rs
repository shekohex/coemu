use crate::{ActorState, Error};
use async_trait::async_trait;
use network::Actor;

/// This trait defines a screen object that may or may not have an owner.
#[async_trait]
pub trait ScreenObject {
    /// The unique identifier of the object.
    fn id(&self) -> usize;
    /// The current X coordinate of the object from the spawn packet.
    fn x(&self) -> u16;
    /// The current Y coordinate of the object from the spawn packet.
    fn y(&self) -> u16;
    /// This method sends the character's spawn packet to another player. It is
    /// called by the screen system when the players appear in each others'
    /// screens. By default, the actor of the screen change loads the spawn
    /// data for both players.
    fn send_spawn(&self, to: &Actor<ActorState>) -> Result<(), Error>;
    /// The ownership of the object (an actor on the server for example).
    fn owner(&self) -> Option<&Actor<ActorState>> { None }
}
