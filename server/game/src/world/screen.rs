use crate::{
    packets::{ActionType, MsgAction},
    ActorState, Error,
};
use async_trait::async_trait;
use dashmap::DashMap;
use std::{fmt::Debug, sync::Arc};
use tq_network::{Actor, PacketEncode};
use tracing::warn;

/// This struct encapsulates the client's screen system. It handles screen
/// objects that the player can currently see in the client window as they
/// enter, move, and leave the screen. It controls the distribution of packets
/// to the other players in the screen and adding new objects as the character
/// (the actor) moves.
#[derive(Clone, Default, Debug)]
pub struct Screen {
    owner: Option<Actor<ActorState>>,
    objects: Arc<DashMap<usize, Box<dyn ScreenObject>>>,
    actors: Arc<DashMap<usize, Box<dyn ScreenObject>>>,
}

impl Screen {
    pub fn new(owner: Actor<ActorState>) -> Self {
        Self {
            owner: Some(owner),
            ..Default::default()
        }
    }

    pub fn clear(&self) {
        self.objects.clear();
        self.actors.clear();
    }

    /// This method adds the screen object specified in the parameter arguments
    /// to the owner's screen. If the object already exists in the screen,
    /// it will not be added and this method will return false. If the
    /// screen object is being added, and the object is of type character, then
    /// the owner will be added to the observer's screen as well.
    pub async fn insert(
        &self,
        obj: Box<dyn ScreenObject>,
    ) -> Result<bool, Error> {
        // If the object is a character, add the owner to the observer (if the
        // owner is visible).
        let observer = obj.owner();
        if obj.is_charachter() {
            let added = self.actors.insert(obj.id(), obj).is_none();
            match (&observer, &self.owner) {
                (Some(observer), Some(owner)) if added => {
                    let observer_screen = observer.screen().await?;
                    let character = owner.character().await?;
                    let res = observer_screen
                        .actors
                        .insert(owner.id(), Box::new(character))
                        .is_none();
                    Ok(res)
                },
                _ => Ok(false),
            }
        } else {
            let res = self.objects.insert(obj.id(), obj).is_none();
            Ok(res)
        }
    }

    /// This method removes a screen object from the owner's screen without
    /// using force. It will not remove the spawn. This method is used for
    /// characters who are actively removing themselves out of the screen.
    pub async fn remove(&self, id: usize) -> Result<bool, Error> {
        let maybe_obj = self.actors.remove(&id);
        if let Some((_, obj)) = maybe_obj {
            match (&obj.owner(), &self.owner) {
                (Some(observer), Some(owner)) => {
                    let observer_screen = observer.screen().await?;
                    let removed =
                        observer_screen.actors.remove(&owner.id()).is_some();
                    Ok(removed)
                },
                _ => Ok(true),
            }
        } else {
            let removed = self.objects.remove(&id).is_some();
            Ok(removed)
        }
    }

    /// This method deletes a screen object from the owner's screen. It uses the
    /// entity removal subtype from
    /// the general action packet to forcefully remove the entity from the
    /// owner's screen. It returns false if the character was never in the
    /// owner's screen to begin with.
    pub async fn delete(&self, id: usize) -> Result<bool, Error> {
        let observer = match (self.actors.remove(&id), self.objects.remove(&id))
        {
            (Some((_, actor)), None) => actor,
            (None, Some((_, object))) => object,
            _ => {
                return Ok(false);
            },
        };
        if let Some(owner) = &self.owner {
            owner
                .send(MsgAction::new(
                    observer.id() as u32,
                    0,
                    0,
                    0,
                    ActionType::RemoveEntity,
                ))
                .await?;
        }
        Ok(true)
    }

    /// This method removes the owner from all observers. It makes use of the
    /// delete method (general action subtype packet) to forcefully remove
    /// the owner from each screen within the owner's screen distance.
    pub async fn remove_from_observers(&self) -> Result<(), Error> {
        for observer in self.actors.iter() {
            match (&observer.owner(), &self.owner) {
                (Some(observer), Some(owner)) => {
                    let observer_screen = observer.screen().await?;
                    observer_screen.delete(owner.id()).await?;
                },
                _ => continue,
            }
        }
        Ok(())
    }

    pub async fn refresh_spawn_for_observers(&self) -> Result<(), Error> {
        for observer in self.actors.iter() {
            match (&observer.owner(), &self.owner) {
                (Some(observer), Some(owner)) => {
                    let observer_screen = observer.screen().await?;
                    observer_screen.delete(owner.id()).await?;
                    let mycharacter = owner.character().await?;
                    mycharacter.send_spawn(observer).await?;
                },
                _ => continue,
            }
        }
        Ok(())
    }

    /// This method loads the character's surroundings from the owner's current
    /// map after a teleportation. It iterates through each map object and
    /// spawns it to the owner's screen (if the object is within the owner's
    /// screen distance).
    pub async fn load_surroundings(&self) -> Result<(), Error> {
        // Load Players from the Map
        if let Some(owner) = &self.owner {
            let mymap = owner.map().await?;
            for observer in mymap.actors().iter() {
                let is_myself = owner.id() == observer.id();
                if is_myself {
                    continue;
                }
                let observer_character = observer.character().await?;
                let mycharacter = owner.character().await?;
                let in_screen = tq_math::in_screen(
                    (observer_character.x(), observer_character.y()),
                    (mycharacter.x(), mycharacter.y()),
                );
                if !in_screen {
                    continue;
                }
                let added =
                    self.insert(Box::new(observer_character.clone())).await?;
                if added {
                    mycharacter
                        .exchange_spawn_packets(observer_character)
                        .await?;
                }
            }
        } else {
            warn!("Owner is None!");
        }
        Ok(())
    }

    /// act as "send to all" method, this method sends a packet to
    /// each observing client in the owner's screen; however, if the player
    /// is invisible, the message packet will be sent, regardless.
    pub async fn send_message<P: PacketEncode + Clone>(
        &self,
        packet: P,
    ) -> Result<(), P::Error> {
        for observer in self.actors.iter() {
            match observer.owner() {
                Some(observer) => {
                    observer.send(packet.clone()).await?;
                },
                _ => continue,
            }
        }
        Ok(())
    }

    /// This method sends a movement packet to all observers that fall within
    /// the owner's new screen distance. It filters through each player on
    /// the map according to screen distance. If the character is within the
    /// owner's new screen distance, the method will attempt to add the observer
    /// to the owner's screen. If the observer is already in the screen, the
    /// owner will send the movement packet to it. If the observer is not
    /// within the new screen distance, the method will attempt to remove it
    /// from the owner's screen.
    pub async fn send_movement<P: PacketEncode + Clone>(
        &self,
        packet: P,
    ) -> Result<(), Error> {
        if let Some(owner) = &self.owner {
            let mymap = owner.map().await?;
            // For each possible observer on the map
            for observer in mymap.actors().iter() {
                let is_myself = owner.id() == observer.id();
                // skip myself
                if is_myself {
                    continue;
                }
                let observer_character = observer.character().await?;
                let mycharacter = owner.character().await?;
                // If the character is in screen, make sure it's in the owner's
                // screen:
                let in_screen = tq_math::in_screen(
                    (observer_character.x(), observer_character.y()),
                    (mycharacter.x(), mycharacter.y()),
                );
                if in_screen {
                    let added = self
                        .insert(Box::new(observer_character.clone()))
                        .await?;
                    // new, let's exchange spawn packets
                    if added {
                        mycharacter
                            .exchange_spawn_packets(observer_character)
                            .await?;
                    } else {
                        // observer is already there, send the movement packet
                        observer.send(packet.clone()).await.unwrap_or_default();
                    }
                } else {
                    //  Else, remove the observer and send the last packet.
                    let observer_screen = observer.screen().await?;
                    observer_screen.remove(owner.id()).await?;
                    let myscreen = owner.screen().await?;
                    myscreen.remove(observer.id()).await?;
                    // send the last packet.
                    observer.send(packet.clone()).await.unwrap_or_default();
                }
            }
        }
        Ok(())
    }
}

/// This trait defines a screen object that may or may not have an owner.
#[async_trait]
pub trait ScreenObject: Send + Sync + Debug {
    /// The unique identifier of the object.
    fn id(&self) -> usize;
    /// The current X coordinate of the object from the spawn packet.
    fn x(&self) -> u16;
    /// The current Y coordinate of the object from the spawn packet.
    fn y(&self) -> u16;
    /// The ownership of the object (an actor on the server for example).
    fn owner(&self) -> Option<Actor<ActorState>> { None }

    fn is_charachter(&self) -> bool { false }

    /// This method sends the character's spawn packet to another player. It is
    /// called by the screen system when the players appear in each others'
    /// screens. By default, the actor of the screen change loads the spawn
    /// data for both players.
    async fn send_spawn(&self, to: &Actor<ActorState>) -> Result<(), Error>;
}
