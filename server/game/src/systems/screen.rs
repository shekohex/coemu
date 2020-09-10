use crate::{
    entities::BaseEntity,
    packets::{ActionType, MsgAction},
    utils::LoHi,
    world::Character,
    ActorState, Error,
};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::sync::RwLock;
use tq_network::{Actor, PacketEncode};
use tracing::debug;

type Characters = Arc<RwLock<HashMap<u32, Character>>>;
/// This struct encapsulates the client's screen system. It handles screen
/// objects that the player can currently see in the client window as they
/// enter, move, and leave the screen. It controls the distribution of packets
/// to the other players in the screen and adding new objects as the character
/// (the actor) moves.
#[derive(Clone, Default, Debug)]
pub struct Screen {
    owner: Actor<ActorState>,
    characters: Characters,
}

impl Screen {
    pub fn new(owner: Actor<ActorState>) -> Self {
        debug!("Creating Screen for Actor #{}", owner.id());
        Self {
            owner,
            ..Default::default()
        }
    }

    pub async fn clear(&self) -> Result<(), Error> {
        debug!("Clearing Screen..");
        let me = self.owner.character().await?;
        let characters = self.characters.read().await;
        for character in characters.values() {
            let observer = character.owner();
            let observer_screen = observer.screen().await?;
            observer_screen.remove_character(me.id()).await?;
        }
        drop(characters);
        self.characters.write().await.clear();
        debug!("Screen is clean!");
        Ok(())
    }

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
            let me = self.owner.character().await?;
            debug!("Added #{} to #{}", observer.id(), me.id());
            let observer_screen = observer.owner().screen().await?;
            let res = observer_screen
                .characters
                .write()
                .await
                .insert(me.id(), me)
                .is_none();
            let res = added && res;
            Ok(res)
        } else {
            Ok(added)
        }
    }

    pub async fn remove_character(&self, id: u32) -> Result<bool, Error> {
        if let Some(observer_character) =
            self.characters.write().await.remove(&id)
        {
            let me = self.owner.character().await?;
            debug!("Removed #{} from #{}", observer_character.id(), me.id());
            let observer_screen = observer_character.owner().screen().await?;
            let removed = observer_screen
                .characters
                .write()
                .await
                .remove(&me.id())
                .is_some();
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    pub async fn delete_character(&self, id: u32) -> Result<bool, Error> {
        let deleted = self.characters.write().await.remove(&id);
        let me = self.owner.character().await?;
        if let Some(other) = deleted {
            let location = u32::constract(other.y(), other.x());
            self.owner
                .send(MsgAction::new(
                    id,
                    other.map_id(),
                    location,
                    other.direction() as u16,
                    ActionType::LeaveMap,
                ))
                .await?;
            debug!("Deleted #{} from #{}", id, me.id());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// This method removes the owner from all observers. It makes use of the
    /// delete method (general action subtype packet) to forcefully remove
    /// the owner from each screen within the owner's screen distance.
    pub async fn remove_from_observers(&self) -> Result<(), Error> {
        let me = self.owner.character().await?;
        for observer in self.characters.read().await.values() {
            debug!("Found Observer #{}", observer.id());
            let observer_screen = observer.owner().screen().await?;
            observer_screen.delete_character(me.id()).await?;
            debug!(
                "#{} Removed from Observer #{} Screen",
                me.id(),
                observer.id()
            );
        }
        Ok(())
    }

    pub async fn refresh_spawn_for_observers(&self) -> Result<(), Error> {
        let me = self.owner.character().await?;
        for observer in self.characters.read().await.values() {
            let observer_screen = observer.owner().screen().await?;
            observer_screen.delete_character(me.id()).await?;
            me.send_spawn(&observer.owner()).await?;
        }
        Ok(())
    }

    /// This method loads the character's surroundings from the owner's current
    /// map after a teleportation. It iterates through each map object and
    /// spawns it to the owner's screen (if the object is within the owner's
    /// screen distance).
    pub async fn load_surroundings(&self) -> Result<(), Error> {
        // Load Players from the Map
        let mymap = self.owner.map().await?;
        let me = self.owner.character().await?;
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
                debug!("Loaded #{} Into #{} Screen", observer.id(), me.id());
                me.exchange_spawn_packets(observer.clone()).await?;
            }
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
    pub async fn send_movement<P: PacketEncode + Clone>(
        &self,
        packet: P,
    ) -> Result<(), Error> {
        let mymap = self.owner.map().await?;
        // For each possible observer on the map
        let me = self.owner.character().await?;
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
                    debug!(
                        "Loaded #{} Into #{} Screen",
                        observer.id(),
                        me.id()
                    );
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
                let observer_screen = observer_owner.screen().await?;
                let removed = observer_screen.remove_character(me.id()).await?;
                if removed {
                    debug!(
                        "UnLoaded #{} From #{} Screen",
                        me.id(),
                        observer.id(),
                    );
                    // send the last packet.
                    observer_owner
                        .send(packet.clone())
                        .await
                        .unwrap_or_default();
                }
                let myscreen = self.owner.screen().await?;
                let removed = myscreen.remove_character(observer.id()).await?;
                if removed {
                    debug!(
                        "UnLoaded #{} From #{} Screen",
                        observer.id(),
                        me.id(),
                    );
                }
            }
        }
        Ok(())
    }
}
