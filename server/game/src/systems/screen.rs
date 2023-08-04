use crate::entities::BaseEntity;
use crate::packets::{ActionType, MsgAction};
use crate::world::Character;
use crate::Error;
use arc_swap::ArcSwapWeak;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Weak};
use tq_network::{ActorHandle, PacketEncode, PacketID};
use tracing::debug;

type Characters = RwLock<HashMap<u32, Weak<Character>>>;
/// This struct encapsulates the client's screen system. It handles screen
/// objects that the player can currently see in the client window as they
/// enter, move, and leave the screen. It controls the distribution of packets
/// to the other players in the screen and adding new objects as the character
/// (the actor) moves.
#[derive(Debug)]
pub struct Screen {
    owner: ActorHandle,
    character: ArcSwapWeak<Character>,
    characters: Characters,
}

impl Screen {
    pub fn new(owner: ActorHandle) -> Self {
        debug!(owner = owner.id(), "Creating Screen");
        Self {
            owner,
            character: Default::default(),
            characters: RwLock::new(HashMap::new()),
        }
    }

    pub fn try_character(&self) -> Result<Arc<Character>, Error> {
        self.character
            .load()
            .upgrade()
            .ok_or(Error::CharacterNotFound)
    }

    pub fn set_character(&self, character: Weak<Character>) {
        self.character.store(character);
    }

    pub fn with_characters<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<u32, Weak<Character>>) -> R,
    {
        f(&self.characters.read())
    }

    pub fn with_characters_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<u32, Weak<Character>>) -> R,
    {
        f(&mut self.characters.write())
    }

    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub fn clear(&self) -> Result<(), Error> {
        *self.characters.write() = HashMap::new();
        Ok(())
    }

    /// This method adds the screen object specified in the parameter arguments
    /// to the owner's screen. If the object already exists in the screen,
    /// it will not be added and this method will return false. If the
    /// screen object is being added, and the object is of type character, then
    /// the owner will be added to the observer's screen as well.
    #[tracing::instrument(skip(self, observer), fields(me = self.owner.id()))]
    pub fn insert_charcter(
        &self,
        observer: Weak<Character>,
    ) -> Result<bool, Error> {
        let me = self.try_character()?;
        let observer_character =
            observer.upgrade().ok_or(Error::CharacterNotFound)?;
        let observer_id = observer_character.id();
        let added = self.with_characters_mut(|c| {
            c.insert(observer_character.id(), observer).is_none()
        });
        if added {
            debug!(observer = observer_id, "Added to Screen");
            let res = match observer_character.try_screen() {
                Ok(observer_screen) => {
                    observer_screen.with_characters_mut(|c| {
                        c.insert(me.id(), Arc::downgrade(&me)).is_none()
                    })
                },
                Err(_) => false,
            };
            Ok(res)
        } else {
            Ok(false)
        }
    }

    /// This method removes a screen object from the owner's screen without
    /// using force. It will not remove the spawn. This method is used for
    /// characters who are actively removing themselves out of the screen.
    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub fn remove_character(&self, observer: u32) -> Result<bool, Error> {
        let observer_character =
            self.with_characters_mut(|c| c.remove(&observer));
        if let Some(observer) = observer_character.and_then(|c| c.upgrade()) {
            debug!(observer = observer.id(), "Removed from Screen");
            let Ok(observer_screen) = observer.try_screen() else {
                return Ok(false);
            };
            let me = self.try_character()?;
            let removed = observer_screen
                .with_characters_mut(|c| c.remove(&me.id()).is_some());
            Ok(removed)
        } else {
            Ok(false)
        }
    }

    /// This method deletes a screen object from the owner's screen. It uses the
    /// entity removal subtype from the general action packet to forcefully
    /// remove the entity from the owner's screen. It returns false if
    /// the character was never in the owner's screen to begin with.
    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub async fn delete_character(&self, observer: u32) -> Result<bool, Error> {
        let deleted =
            self.with_characters_mut(|c| c.remove(&observer).is_some());
        if deleted {
            self.owner
                .send(MsgAction::new(
                    observer,
                    observer,
                    0,
                    0,
                    ActionType::LeaveMap,
                ))
                .await?;
            tracing::trace!(%observer, "Deleted from Screen");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// This method removes the owner from all observers. It makes use of the
    /// delete method (general action subtype packet) to forcefully remove
    /// the owner from each screen within the owner's screen distance.
    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub async fn remove_from_observers(&self) -> Result<(), Error> {
        let me_id = self.try_character()?.id();
        let futures = FuturesUnordered::new();
        self.with_characters(|c| {
            let iter = c.values().filter_map(|v| v.upgrade());
            for observer in iter {
                tracing::trace!(observer = observer.id(), "Found Observer");
                let Ok(observer_screen) = observer.try_screen() else {
                    continue;
                };
                let fut = async move {
                    observer_screen.delete_character(me_id).await?;
                    Result::<_, Error>::Ok(observer.id())
                };
                futures.push(fut);
            }
        });
        // await all futures to complete.
        futures
            .for_each_concurrent(None, |res| async {
               match res {
                   Ok(observer) => {
                       tracing::trace!(observer = observer, "Removed from Observer's Screen");
                   },
                   Err(e) => {
                       tracing::error!(error = ?e, "Failed to delete from screen");
                   },
               }
            })
            .await;
        // take a moment to clean up any weak references that may have been
        // dropped.
        self.with_characters_mut(|c| {
            c.retain(|_, v| v.upgrade().is_some());
        });
        Ok(())
    }

    /// This method removes the owner from all observers. It makes use of the
    /// delete method (general action subtype packet) to forcefully remove
    /// the owner from each screen within the owner's screen distance.
    /// It then respawns the character in the observers' screens.
    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub async fn refresh_spawn_for_observers(&self) -> Result<(), Error> {
        let me = self.try_character()?;
        let futures = FuturesUnordered::new();
        self.with_characters(|c| {
            let iter = c.values().filter_map(|v| v.upgrade());
            for observer in iter {
                debug!(observer = observer.id(), "Found Observer");
                let observer_owner = observer.owner();
                let Ok(observer_screen) = observer.try_screen() else {
                    continue;
                };
                let me = me.clone();
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
                        tracing::trace!(%observer, "Refreshed Spawn");
                    },
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to refresh spawn");
                    },
                }
            })
            .await;
        // take a moment to clean up any weak references that may have been
        // dropped.
        self.with_characters_mut(|c| {
            c.retain(|_, v| v.upgrade().is_some());
        });

        Ok(())
    }

    /// This method loads the character's surroundings from the owner's current
    /// map after a teleportation. It iterates through each map object and
    /// spawns it to the owner's screen (if the object is within the owner's
    /// screen distance).
    #[tracing::instrument(skip(self, state), fields(me = self.owner.id()))]
    pub async fn load_surroundings(
        &self,
        state: &crate::State,
    ) -> Result<(), Error> {
        // Load Players from the Map
        let me = &self.try_character()?;
        let mymap = state.try_map(me.entity().map_id())?;
        let loc = me.entity().location();
        let myreagions = mymap.surrunding_regions(loc.x, loc.y);
        let futures = FuturesUnordered::new();
        for region in myreagions {
            tracing::trace!(%region, "Loading Surroundings");
            region.with_characters(|c| {
                let iter = c.values().filter_map(|v| v.upgrade());
                for observer in iter {
                    let is_myself = me.id() == observer.id();
                    if is_myself {
                        continue;
                    }
                    let observer_loc = observer.entity().location();
                    let in_screen = tq_math::in_screen(
                        (observer_loc.x, observer_loc.y),
                        (loc.x, loc.y),
                    );
                    if !in_screen {
                        continue;
                    }
                    let observer = observer.clone();
                    let fut = async move {
                        let added =
                            self.insert_charcter(Arc::downgrade(&observer))?;
                        if added {
                            tracing::trace!(
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
    #[tracing::instrument(skip(self, packet), fields(me = self.owner.id(), packet_id = P::PACKET_ID))]
    pub async fn send_message<P>(&self, packet: P) -> Result<(), P::Error>
    where
        P: PacketEncode + PacketID + Clone,
    {
        let futures = FuturesUnordered::new();
        self.with_characters(|c| {
            let iter = c.values().filter_map(|v| v.upgrade());
            for observer in iter {
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
        // take a moment to clean up any weak references that may have been
        // dropped.
        self.with_characters_mut(|c| {
            c.retain(|_, v| v.upgrade().is_some());
        });
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
    #[tracing::instrument(skip(self, state, packet), fields(me = self.owner.id(), packet_id = P::PACKET_ID))]
    pub async fn send_movement<P>(
        &self,
        state: &crate::State,
        packet: P,
    ) -> Result<(), Error>
    where
        P: PacketEncode + PacketID + Clone + Send + Sync + 'static,
    {
        let me = &self.try_character()?;
        let mymap = state.try_map(me.entity().map_id())?;
        let loc = me.entity().location();
        let myreagions = mymap.surrunding_regions(loc.x, loc.y);
        let futures = FuturesUnordered::new();
        for region in myreagions {
            tracing::trace!(%region, "Sending Movement");
            region.with_characters(|c| {
                // For each possible observer on the region:
                let iter = c.values().filter_map(|v| v.upgrade());
                for observer in iter {
                    let is_myself = me.id() == observer.id();
                    let observer_owner = observer.owner();
                    // skip myself
                    if is_myself {
                        continue;
                    }
                    // If the character is in screen, make sure it's in the
                    // owner's screen:
                    let observer_loc = observer.entity().location();
                    let in_screen = tq_math::in_screen(
                        (observer_loc.x, observer_loc.y),
                        (loc.x, loc.y),
                    );
                    if in_screen {
                        let packet = packet.clone();
                        let observer = observer.clone();
                        let fut = async move {
                            let added = self
                                .insert_charcter(Arc::downgrade(&observer))?;
                            // new, let's exchange spawn packets
                            if added {
                                tracing::trace!(
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
                                tracing::trace!(
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
                                tracing::trace!(
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
        // take a moment to clean up any weak references that may have been
        // dropped.
        self.with_characters_mut(|c| {
            c.retain(|_, v| v.upgrade().is_some());
        });
        Ok(())
    }
}
