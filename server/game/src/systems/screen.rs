use crate::entities::GameEntity;
use crate::packets::{ActionType, MsgAction};
use crate::Error;
use arc_swap::ArcSwapWeak;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use parking_lot::RwLock;
use primitives::Location;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Weak};
use tq_network::{ActorHandle, PacketEncode, PacketID};
use tracing::debug;

type Entities = RwLock<HashMap<u32, Weak<GameEntity>>>;
/// This struct encapsulates the client's screen system. It handles screen
/// objects that the player can currently see in the client window as they
/// enter, move, and leave the screen. It controls the distribution of packets
/// to the other players in the screen and adding new objects as the character
/// (the actor) moves.
#[derive(Debug)]
pub struct Screen {
    owner: ActorHandle,
    character: ArcSwapWeak<GameEntity>,
    entities: Entities,
}

impl Screen {
    pub fn new(owner: ActorHandle) -> Self {
        Self {
            owner,
            character: Default::default(),
            entities: Default::default(),
        }
    }

    /// This method returns the character id of the owner of the screen.
    pub fn character_id(&self) -> Result<u32, Error> {
        self.character
            .load()
            .upgrade()
            .ok_or(Error::CharacterNotFound)
            .map(|c| c.id())
    }

    pub fn set_character(&self, character: Weak<GameEntity>) {
        self.character.store(character);
    }

    pub fn with_entities<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&HashMap<u32, Weak<GameEntity>>) -> R,
    {
        f(&self.entities.read())
    }

    pub fn with_entities_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HashMap<u32, Weak<GameEntity>>) -> R,
    {
        f(&mut self.entities.write())
    }

    /// This method clears the screen of all entities. It is used when the
    /// character is logging out or changing maps.
    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub fn clear(&self) -> Result<(), Error> {
        // Clear the entities in the screen.
        *self.entities.write() = HashMap::new();
        Ok(())
    }

    /// This method adds the screen object specified in the parameter arguments
    /// to the owner's screen. If the object already exists in the screen,
    /// it will not be added and this method will return false. If the
    /// screen object is being added, and the object is of type character, then
    /// the owner will be added to the observer's screen as well.
    #[tracing::instrument(skip(self, observer), fields(me = self.owner.id()))]
    pub fn insert_entity(&self, observer: Weak<GameEntity>) -> Result<bool, Error> {
        let o = observer.upgrade().ok_or(Error::CharacterNotFound)?;
        let alredy_exists = self.with_entities(|c| c.contains_key(&o.id()));
        if alredy_exists {
            return Ok(false);
        }
        let oid = o.id();
        let added = self.with_entities_mut(|c| c.insert(oid, Arc::downgrade(&o)).is_none());
        if !added {
            return Ok(false);
        }
        match o.as_ref() {
            GameEntity::Character(c) => {
                let me = self.character.load().upgrade().ok_or(Error::CharacterNotFound)?;
                debug!(character = c.id(), "Added Character to Screen");
                let oscreen = c.try_screen()?;
                let myid = self.character_id()?;
                let added = oscreen.with_entities_mut(|c| c.insert(myid, Arc::downgrade(&me)).is_none());
                Ok(added)
            },
            GameEntity::Npc(o) => {
                debug!(npc = o.id(), "Added Npc to Screen");
                Ok(true)
            },
        }
    }

    /// This method removes a screen object from the owner's screen without
    /// using force. It will not remove the spawn. This method is used for
    /// characters who are actively removing themselves out of the screen.
    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub fn remove_entity(&self, observer: u32) -> Result<bool, Error> {
        let Some(o) = self.with_entities_mut(|c| c.remove(&observer).and_then(|c| c.upgrade())) else {
            return Ok(false);
        };
        match o.as_ref() {
            GameEntity::Character(c) => {
                debug!(character = c.id(), "Removed Character from Screen");
                let oscreen = c.try_screen()?;
                let myid = self.character_id()?;
                let removed = oscreen.with_entities_mut(|c| c.remove(&myid).is_some());
                Ok(removed)
            },
            GameEntity::Npc(o) => {
                debug!(npc = o.id(), "Removed Npc from Screen");
                Ok(true)
            },
        }
    }

    /// This method deletes a screen object from the owner's screen. It uses the
    /// entity removal subtype from the general action packet to forcefully
    /// remove the entity from the owner's screen. It returns false if
    /// the character was never in the owner's screen to begin with.
    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub async fn delete_character(&self, observer: u32) -> Result<bool, Error> {
        let deleted = self.with_entities_mut(|c| c.remove(&observer).is_some());
        if !deleted {
            return Ok(false);
        }
        self.owner
            .send(MsgAction::new(observer, observer, 0, 0, ActionType::LeaveMap))
            .await?;
        tracing::trace!(%observer, "Deleted from Screen");
        Ok(true)
    }

    /// This method removes the owner from all observers. It makes use of the
    /// delete method (general action subtype packet) to forcefully remove
    /// the owner from each screen within the owner's screen distance.
    #[tracing::instrument(skip(self), fields(me = self.owner.id()))]
    pub async fn remove_from_observers(&self) -> Result<(), Error> {
        let me_id = self.character_id()?;
        let mut tasks = tokio::task::JoinSet::new();
        self.with_entities(|c| {
            let iter = c.values().filter_map(|v| v.upgrade());
            for o in iter {
                match o.as_ref() {
                    GameEntity::Character(c) => {
                        tracing::trace!(character = c.id(), "Found Character Observer");
                        let Ok(screen) = c.try_screen() else {
                            continue;
                        };
                        let fut = async move {
                            screen.delete_character(me_id).await?;
                            Result::<_, Error>::Ok(o.id())
                        };
                        tasks.spawn(fut);
                    },
                    GameEntity::Npc(_) => {
                        tracing::trace!(npc = o.id(), "Found Npc Observer");
                        // Npc's don't need to be removed from the screen.
                        // They are removed when the npc is removed from the
                        // map.
                        continue;
                    },
                }
            }
        });

        while let Some(task) = tasks.join_next().await {
            let res = task?;
            match res {
                Ok(observer) => {
                    tracing::trace!(observer = observer, "Removed from Observer's Screen");
                },
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to delete from screen");
                },
            }
        }
        // take a moment to clean up any weak references that may have been
        // dropped.
        self.with_entities_mut(|c| {
            c.retain(|_, v| v.upgrade().is_some());
        });
        Ok(())
    }

    /// This method loads the character's surroundings from the owner's current
    /// map after a teleportation. It iterates through each map object and
    /// spawns it to the owner's screen (if the object is within the owner's
    /// screen distance).
    #[tracing::instrument(skip(self, state), fields(me = self.owner.id()))]
    pub async fn load_surroundings(&self, state: &crate::State) -> Result<(), Error> {
        let entity = self.character.load().upgrade().ok_or(Error::CharacterNotFound)?;
        let me = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let mymap = state.try_map(me.entity().map_id())?;
        let loc = me.entity().location();
        let myreagions = mymap.surrunding_regions(loc.x, loc.y);
        let futures = FuturesUnordered::new();
        for region in myreagions {
            tracing::trace!(%region, "Loading Surroundings");
            if region.is_empty() {
                continue;
            }
            let myself = entity.clone();
            region.with_entities(|c| {
                let iter = c.values().filter_map(|v| v.upgrade());
                for o in iter {
                    match o.as_ref() {
                        GameEntity::Character(c) if c.id() == me.id() => {
                            continue;
                        },
                        GameEntity::Character(_) if can_see(&o, &myself) => {
                            let o = o.clone();
                            let fut = async move {
                                let added = self.insert_entity(Arc::downgrade(&o))?;
                                if !added {
                                    return Ok(());
                                }
                                tracing::trace!(character = o.id(), "Loaded Into Screen");
                                me.exchange_spawn_packets(&o).await?;
                                Result::<_, Error>::Ok(())
                            }
                            .boxed();
                            futures.push(fut);
                        },
                        GameEntity::Npc(_) => {
                            let o = o.clone();
                            let me = entity.clone();
                            // Spawn the npc to the owner's screen.
                            let fut = async move {
                                let added = self.insert_entity(Arc::downgrade(&o))?;
                                if !added {
                                    return Ok(());
                                }
                                tracing::trace!(npc = o.id(), "Loaded Into Screen");
                                o.send_spawn(&me).await?;
                                Result::<_, Error>::Ok(())
                            }
                            .boxed();
                            futures.push(fut);
                        },
                        GameEntity::Character(_) => {
                            // Characters that are not in the owner's screen
                            // distance are not loaded into the screen.
                            continue;
                        },
                    }
                }
            });
        }

        futures
            .for_each_concurrent(None, |res| async {
                match res {
                    Ok(_) => {},
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to load entity");
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
        self.with_entities(|c| {
            let iter = c.values().filter_map(|v| v.upgrade().and_then(|o| o.owner()));
            for o in iter {
                let packet = packet.clone();
                let fut = async move {
                    o.send(packet).await?;
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
        self.with_entities_mut(|c| {
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
    pub async fn send_movement<P>(&self, state: &crate::State, packet: P) -> Result<(), Error>
    where
        P: PacketEncode + PacketID + Clone + Send + Sync + 'static,
    {
        let entity = self.character.load().upgrade().ok_or(Error::CharacterNotFound)?;
        let me = entity.as_character().ok_or(Error::CharacterNotFound)?;
        let mymap = state.try_map(me.entity().map_id())?;
        let loc = me.entity().location();
        let myreagions = mymap.surrunding_regions(loc.x, loc.y);
        let futures = FuturesUnordered::new();
        for region in myreagions {
            if region.is_empty() {
                continue;
            }
            region.with_entities(|c| {
                // For each possible observer on the region:
                let iter = c.values().filter_map(|v| v.upgrade());
                for o in iter {
                    let myself = entity.clone();
                    match o.as_ref() {
                        GameEntity::Character(c) if c.id() == me.id() => {
                            continue;
                        },
                        GameEntity::Character(c) if can_see(&o, &myself) => {
                            let packet = packet.clone();
                            let o = o.clone();
                            let oowner = c.owner();
                            let fut = async move {
                                let added = self.insert_entity(Arc::downgrade(&o))?;
                                // new, let's exchange spawn packets
                                if added {
                                    tracing::trace!(observer = o.id(), "Loaded Into Screen",);
                                    me.exchange_spawn_packets(&o).await?;
                                } else {
                                    // observer is already there, send the
                                    // movement
                                    // packet
                                    let _ = oowner.send(packet).await;
                                }
                                Result::<_, Error>::Ok(())
                            }
                            .boxed();
                            futures.push(fut);
                        },
                        GameEntity::Character(c) => {
                            let packet = packet.clone();
                            let oowner = c.owner();
                            let observer_id = o.id();
                            let Ok(oscreen) = c.try_screen() else {
                                continue;
                            };
                            let fut = async move {
                                // Else, remove the observer and send the last
                                // packet.
                                if oscreen.remove_entity(me.id())? {
                                    tracing::trace!(observer = observer_id, "UnLoaded Screen");
                                    // send the last packet.
                                    oowner.send(packet).await.unwrap_or_default();
                                }
                                if self.remove_entity(observer_id)? {
                                    tracing::trace!(observer = observer_id, "Removed from Screen");
                                }
                                Result::<_, Error>::Ok(())
                            }
                            .boxed();
                            futures.push(fut);
                        },
                        GameEntity::Npc(_) if can_see_npc(&o, &myself) => {
                            let fut = async move {
                                let added = self.insert_entity(Arc::downgrade(&o))?;
                                // new, Send NPC spawn packet to the owner
                                if added {
                                    tracing::trace!(npc = o.id(), "Loaded Into Screen");
                                    o.send_spawn(&myself).await?;
                                } else {
                                    // observer is already there
                                    // Do nothing.
                                }
                                Result::<_, Error>::Ok(())
                            }
                            .boxed();
                            futures.push(fut);
                        },
                        GameEntity::Npc(_) => {
                            // NPC not loaded in the screen.
                            // Remove it from the screen.
                            let _ = self.remove_entity(o.id());
                        },
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
        self.with_entities_mut(|c| {
            c.retain(|_, v| v.upgrade().is_some());
        });
        Ok(())
    }
}

fn can_see(a: &GameEntity, b: &GameEntity) -> bool {
    let Location { x: x1, y: y1, .. } = a.basic().location();
    let Location { x: x2, y: y2, .. } = b.basic().location();
    tq_math::in_screen((x1, y1), (x2, y2))
}

fn can_see_npc(a: &GameEntity, b: &GameEntity) -> bool {
    let Location { x: x1, y: y1, .. } = a.basic().location();
    let Location { x: x2, y: y2, .. } = b.basic().location();
    tq_math::in_range((x1, y1), (x2, y2), 16)
}
