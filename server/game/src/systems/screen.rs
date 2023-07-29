use crate::entities::BaseEntity;
use crate::packets::{ActionType, MsgAction};
use crate::utils::LoHi;
use crate::world::Character;
use crate::Error;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tq_network::{ActorHandle, PacketEncode, PacketID};
use tracing::debug;

type Characters = RwLock<HashMap<u32, Arc<Character>>>;
/// This struct encapsulates the client's screen system. It handles screen
/// objects that the player can currently see in the client window as they
/// enter, move, and leave the screen. It controls the distribution of packets
/// to the other players in the screen and adding new objects as the character
/// (the actor) moves.
#[derive(Debug)]
pub struct Screen {
    owner: ActorHandle,
    character: Arc<Character>,
    characters: Characters,
}

impl Screen {
    pub fn new(owner: ActorHandle, character: Arc<Character>) -> Self {
        debug!(
            owner = owner.id(),
            character = character.id(),
            "Creating Screen"
        );
        Self {
            owner,
            character,
            characters: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_characters<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<u32, Arc<Character>>) -> R,
    {
        f(&self.characters.read())
    }

    pub fn with_characters_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<u32, Arc<Character>>) -> R,
    {
        f(&mut self.characters.write())
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub fn clear(&self) -> Result<(), Error> {
        *self.characters.write() = HashMap::new();
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id(), observer = observer.id()))]
    pub fn insert_charcter(
        &self,
        observer: Arc<Character>,
    ) -> Result<bool, Error> {
        let observer_id = observer.id();
        let me_id = self.character.id();
        let added = self.with_characters_mut(|c| {
            c.insert(observer.id(), observer.clone()).is_none()
        });
        if added {
            debug!(observer = observer_id, "Added to Screen");
            let res = match observer.try_screen() {
                Ok(observer_screen) => {
                    observer_screen.with_characters_mut(|c| {
                        c.insert(me_id, self.character.clone()).is_none()
                    })
                },
                Err(_) => false,
            };
            Ok(res)
        } else {
            Ok(false)
        }
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub fn remove_character(&self, observer: u32) -> Result<bool, Error> {
        let observer_character =
            self.with_characters_mut(|c| c.remove(&observer));
        if let Some(other) = observer_character {
            debug!(observer = other.id(), "Removed from Screen");
            let Ok(observer_screen) = other.try_screen() else {
                return Ok(false);
            };
            let removed = observer_screen.with_characters_mut(|c| {
                c.remove(&self.character.id()).is_some()
            });
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub async fn delete_character(&self, observer: u32) -> Result<bool, Error> {
        let deleted = self.with_characters_mut(|c| c.remove(&observer));
        if let Some(other) = deleted {
            let location = u32::constract(other.y(), other.x());
            self.owner
                .send(MsgAction::new(
                    other.id(),
                    other.map_id(),
                    location,
                    other.direction() as u16,
                    ActionType::RemoveEntity,
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
        let futures = FuturesUnordered::new();
        self.with_characters(|c| {
            for observer in c.values() {
                debug!(observer = observer.id(), "Found Observer");
                let observer_owner = observer.owner();
                let Ok(observer_screen) = observer.try_screen() else {
                    continue;
                };
                let fut = async move {
                    observer_screen.delete_character(me.id()).await?;
                    Result::<_, Error>::Ok(observer_owner.id())
                };
                futures.push(fut);
            }
        });
        // await all futures to complete.
        futures
            .for_each_concurrent(None, |res| async {
               match res {
                   Ok(observer) => {
                       debug!(observer = observer, "Removed from Observer's Screen");
                   },
                   Err(e) => {
                       tracing::error!(error = ?e, "Failed to delete from screen");
                   },
               }
            })
            .await;
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(me = self.character.id()))]
    pub async fn refresh_spawn_for_observers(&self) -> Result<(), Error> {
        let me = &self.character;
        let futures = FuturesUnordered::new();
        self.with_characters(|c| {
            for observer in c.values() {
                debug!(observer = observer.id(), "Found Observer");
                let observer_owner = observer.owner();
                let Ok(observer_screen) = observer.try_screen() else {
                    continue;
                };
                let fut = async move {
                    observer_screen.delete_character(me.id()).await?;
                    me.send_spawn(&observer_owner).await?;
                    Result::<_, Error>::Ok(observer_owner.id())
                };
                futures.push(fut);
            }
        });

        // await all futures to complete.
        futures
            .for_each_concurrent(None, |res| async {
                match res {
                    Ok(observer) => {
                        debug!(%observer, "Refreshed Spawn");
                    },
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to refresh spawn");
                    },
                }
            })
            .await;
        // let observer_screen = observer.owner().screen().await;
        // observer_screen.delete_character(me.id()).await?;
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
        let mymap = state.try_map(me.map_id())?;
        let myreagions = mymap.surrunding_regions(me.x(), me.y());
        let futures = FuturesUnordered::new();
        for region in myreagions {
            debug!(%region, "Loading Surroundings");
            region.with_characters(|c| {
                for observer in c.values() {
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
                    let observer = observer.clone();
                    let fut = async move {
                        let added = self.insert_charcter(observer.clone())?;
                        if added {
                            debug!(
                                observer = observer.id(),
                                "Loaded Into Screen"
                            );
                            me.exchange_spawn_packets(observer).await?;
                        }
                        Result::<_, Error>::Ok(())
                    };
                    futures.push(fut);
                }
            });
        }

        // await all futures to complete.
        futures
            .for_each_concurrent(None, |res| async {
                match res {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to load surroundings");
                    },
                }
            })
            .await;
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
        let futures = FuturesUnordered::new();
        self.with_characters(|c| {
            for observer in c.values() {
                let observer_owner = observer.owner();
                let packet = packet.clone();
                let fut = async move {
                    observer_owner.send(packet).await?;
                    Result::<_, P::Error>::Ok(())
                };
                futures.push(fut);
            }
        });
        // await all futures to complete.
        futures
            .for_each_concurrent(None, |res| async {
                match res {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to send message");
                    },
                }
            })
            .await;
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
        P: PacketEncode + PacketID + Clone + Send + Sync + 'static,
    {
        let me = &self.character;
        let mymap = state.maps().get(&me.map_id()).ok_or(Error::MapNotFound)?;
        let myreagions = mymap.surrunding_regions(me.x(), me.y());
        let futures = FuturesUnordered::new();
        for region in myreagions {
            debug!(%region, "Sending Movement");
            region.with_characters(|c| {
                // For each possible observer on the region:
                for observer in c.values() {
                    let is_myself = me.id() == observer.id();
                    let observer_owner = observer.owner();
                    // skip myself
                    if is_myself {
                        continue;
                    }
                    // If the character is in screen, make sure it's in the
                    // owner's screen:
                    let in_screen = tq_math::in_screen(
                        (observer.x(), observer.y()),
                        (me.x(), me.y()),
                    );
                    if in_screen {
                        let packet = packet.clone();
                        let observer = observer.clone();
                        let fut = async move {
                            let added =
                                self.insert_charcter(observer.clone())?;
                            // new, let's exchange spawn packets
                            if added {
                                debug!(
                                    observer = observer.id(),
                                    "Loaded Into Screen",
                                );
                                me.exchange_spawn_packets(observer).await?;
                            } else {
                                // observer is already there, send the movement
                                // packet
                                observer_owner
                                    .send(packet)
                                    .await
                                    .unwrap_or_default();
                            }
                            Result::<_, Error>::Ok(())
                        }
                        .boxed();
                        futures.push(fut);
                    } else {
                        let packet = packet.clone();
                        let observer_owner = observer.owner();
                        let observer_id = observer.id();
                        let Ok(observer_screen) = observer.try_screen() else {
                            continue;
                        };
                        let fut = async move {
                            // Else, remove the observer and send the last
                            // packet.
                            let removed =
                                observer_screen.remove_character(me.id())?;
                            if removed {
                                debug!(
                                    observer = observer_id,
                                    "UnLoaded Screen"
                                );
                                // send the last packet.
                                observer_owner
                                    .send(packet)
                                    .await
                                    .unwrap_or_default();
                            }
                            let removed = self.remove_character(observer_id)?;
                            if removed {
                                debug!(
                                    observer = observer_id,
                                    "Removed from Screen"
                                );
                            }
                            Result::<_, Error>::Ok(())
                        }
                        .boxed();
                        futures.push(fut);
                    }
                }
            });
        }
        // await all futures to complete.
        futures
            .for_each_concurrent(None, |res| async {
                match res {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to send movement");
                    },
                }
            })
            .await;
        Ok(())
    }
}
