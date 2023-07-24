use crate::entities::BaseEntity;
use crate::packets::{ActionType, MsgAction};
use crate::utils::LoHi;
use crate::world::Character;
use crate::Error;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use tq_network::{ActorHandle, PacketEncode, PacketID};
use tracing::debug;

type Characters = Arc<RwLock<HashMap<u32, Character>>>;
/// This struct encapsulates the client's screen system. It handles screen
/// objects that the player can currently see in the client window as they
/// enter, move, and leave the screen. It controls the distribution of packets
/// to the other players in the screen and adding new objects as the character
/// (the actor) moves.
#[derive(Clone, Debug)]
pub struct Screen {
    owner: ActorHandle,
    character: Character,
    characters: Characters,
}

impl Screen {
    pub fn new(owner: ActorHandle, character: Character) -> Self {
        debug!("Creating Screen for Actor #{}", owner.id());
        Self {
            owner,
            character,
            characters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub async fn clear(&self) -> Result<(), Error> {
        debug!("Clearing Screen..");
        self.characters.write().await.clear();
        debug!("Screen is clean!");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id(), observer = observer.id()))]
    pub async fn insert_charcter(
        &self,
        observer: Character,
    ) -> Result<bool, Error> {
        let added = self
            .characters
            .write()
            .await
            .insert(observer.id(), observer.clone())
            .is_none();
        if added {
            debug!(observer = observer.id(), "Added to Screen");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub async fn remove_character(&self, observer: u32) -> Result<bool, Error> {
        if self.characters.write().await.remove(&observer).is_some() {
            debug!(%observer, "Removed from Screen");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub async fn delete_character(&self, observer: u32) -> Result<bool, Error> {
        let deleted = self.characters.write().await.remove(&observer);
        if let Some(other) = deleted {
            let location = u32::constract(other.y(), other.x());
            self.owner
                .send(MsgAction::new(
                    other.id(),
                    other.map_id(),
                    location,
                    other.direction() as u16,
                    ActionType::LeaveMap,
                ))
                .await?;
            debug!(%observer, "Deleted from Screen");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// This method removes the owner from all observers. It makes use of the
    /// delete method (general action subtype packet) to forcefully remove
    /// the owner from each screen within the owner's screen distance.
    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub async fn remove_from_observers(&self) -> Result<(), Error> {
        let me = &self.character;
        for observer in self.characters.read().await.values() {
            debug!(observer = observer.id(), "Found Observer");
            // let observer_screen = observer.owner().screen().await;
            // observer_screen.delete_character(me.id()).await?;
            let location = u32::constract(me.y(), me.x());
            observer
                .owner()
                .send(MsgAction::new(
                    me.id(),
                    me.map_id(),
                    location,
                    me.direction() as u16,
                    ActionType::LeaveMap,
                ))
                .await?;
            debug!(observer = observer.id(), "Removed from Observer Screen");
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub async fn refresh_spawn_for_observers(&self) -> Result<(), Error> {
        let me = &self.character;
        for observer in self.characters.read().await.values() {
            // let observer_screen = observer.owner().screen().await;
            // observer_screen.delete_character(me.id()).await?;
            let location = u32::constract(me.y(), me.x());
            observer
                .owner()
                .send(MsgAction::new(
                    me.id(),
                    me.map_id(),
                    location,
                    me.direction() as u16,
                    ActionType::LeaveMap,
                ))
                .await?;
            me.send_spawn(&observer.owner()).await?;
            debug!(observer = observer.id(), "Refreshed Spawn");
        }
        Ok(())
    }

    /// This method loads the character's surroundings from the owner's current
    /// map after a teleportation. It iterates through each map object and
    /// spawns it to the owner's screen (if the object is within the owner's
    /// screen distance).
    #[tracing::instrument(skip(self, state), fields(me = self.character.id()))]
    pub async fn load_surroundings(
        &self,
        state: &crate::State,
    ) -> Result<(), Error> {
        // Load Players from the Map
        let me = &self.character;
        let mymap = state.maps().get(&me.map_id()).ok_or(Error::MapNotFound)?;
        for observer in mymap.characters().read().await.values() {
            let is_myself = me.id() == observer.id();
            if is_myself {
                continue;
            }
            let in_screen = tq_math::in_screen(
                (observer.x(), observer.y()),
                (me.x(), me.y()),
            );
            if !in_screen {
                continue;
            }
            let added = self.insert_charcter(observer.clone()).await?;
            if added {
                debug!(observer = observer.id(), "Loaded Into Screen");
                me.exchange_spawn_packets(observer.clone()).await?;
            }
        }
        Ok(())
    }

    /// act as "send to all" method, this method sends a packet to
    /// each observing client in the owner's screen; however, if the player
    /// is invisible, the message packet will be sent, regardless.
    #[tracing::instrument(skip(self, packet), fields(me = self.character.id(), packet_id = P::id()))]
    pub async fn send_message<P>(&self, packet: P) -> Result<(), P::Error>
    where
        P: PacketEncode + PacketID + Clone,
    {
        for observer in self.characters.read().await.values() {
            observer.owner().send(packet.clone()).await?;
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
    #[tracing::instrument(skip(self, state, packet), fields(me = self.character.id(), packet_id = P::id()))]
    pub async fn send_movement<P>(
        &self,
        state: &crate::State,
        packet: P,
    ) -> Result<(), Error>
    where
        P: PacketEncode + PacketID + Clone,
    {
        let me = &self.character;
        let mymap = state.maps().get(&me.map_id()).ok_or(Error::MapNotFound)?;
        // For each possible observer on the map
        for observer in mymap.characters().read().await.values() {
            let is_myself = me.id() == observer.id();
            let observer_owner = observer.owner();
            // skip myself
            if is_myself {
                continue;
            }
            // If the character is in screen, make sure it's in the owner's
            // screen:
            let in_screen = tq_math::in_screen(
                (observer.x(), observer.y()),
                (me.x(), me.y()),
            );
            if in_screen {
                let added = self.insert_charcter(observer.clone()).await?;
                // new, let's exchange spawn packets
                if added {
                    debug!(observer = observer.id(), "Loaded Into Screen",);
                    me.exchange_spawn_packets(observer.clone()).await?;
                } else {
                    // observer is already there, send the movement packet
                    observer_owner
                        .send(packet.clone())
                        .await
                        .unwrap_or_default();
                }
            } else {
                //  Else, remove the observer and send the last packet.
                debug!(observer = observer.id(), "UnLoaded Screen");
                // send the last packet.
                observer_owner
                    .send(packet.clone())
                    .await
                    .unwrap_or_default();
                let removed = self.remove_character(observer.id()).await?;
                if removed {
                    debug!(observer = observer.id(), "Removed from Screen");
                }
            }
        }
        Ok(())
    }
}
