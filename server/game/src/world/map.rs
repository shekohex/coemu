use super::{floor::Floor, ScreenObject};
use crate::{db, ActorState, Error};
use dashmap::DashMap;
use primitives::Point;
use std::sync::Arc;
use tokio::sync::RwLock;
use tq_network::Actor;

/// This struct encapsulates map information from a compressed map and the
/// database. It includes the identification of the map, pools and methods for
/// character tracking and screen updates, and other methods for processing map
/// actions and events. It composes the floor struct which defines the map's
/// coordinate tile grid.
#[derive(Debug, Default, Clone)]
pub struct Map {
    /// The Inner map loaded from the database
    inner: Arc<db::Map>,
    /// Holds client information for each player on the map.
    actors: Arc<DashMap<usize, Actor<ActorState>>>,
    /// where should the player get revived on this map
    revive_point: Arc<Point<u32>>,
    /// defines the map's coordinate tile grid.
    floor: Arc<RwLock<Floor>>,
}

impl Map {
    pub fn new(inner: db::Map) -> Self {
        Self {
            floor: Arc::new(RwLock::new(Floor::new(inner.path.clone()))),
            actors: Arc::new(DashMap::new()),
            revive_point: Arc::new(Point::new(
                inner.revive_point_x as u32,
                inner.revive_point_y as u32,
            )),
            inner: Arc::new(inner),
        }
    }

    pub fn id(&self) -> u32 { self.inner.map_id as u32 }

    // This method loads a compressed map from the server's flat file database.
    // If the file does not exist, the
    /// server will make an attempt to find and convert a dmap version of the
    /// map into a compressed map file. After converting the map, the map
    /// will be loaded for the server.
    pub async fn load(&mut self) -> Result<(), Error> {
        self.floor.write().await.load().await?;
        Ok(())
    }

    /// This method adds the client specified in the parameters to the map pool.
    /// It does this by removing the player from the previous map, then
    /// adding it to the current map. As the character is added, its map,
    /// current tile, and current elevation are changed.
    pub async fn add_actor(
        &self,
        actor: &Actor<ActorState>,
    ) -> Result<(), Error> {
        {
            // Remove the client from the previous map
            let actor_map = actor.state().map().await;
            actor_map.remove_actor(actor.id()).await?;
        }
        // Add the player to the current map
        let added = self.actors.insert(actor.id(), actor.clone()).is_none();
        if added {
            let mut actor_map = actor.state().map_mut().await;
            *actor_map = self.clone();
            let mut actor_tile = actor.state().tile_mut().await;
            let mut character = actor.state().character_mut().await;
            let floor = self.floor.read().await;
            *actor_tile = floor[(character.x() as u32, character.y() as u32)];
            character.set_elevation(actor_tile.elevation);
            // TODO(shekohex): Send environment packets.
        }
        Ok(())
    }

    /// This method removes the client specified in the parameters from the map.
    /// If the screen of the character still exists, it will remove the
    /// character from each observer's screen.
    pub async fn remove_actor(&self, actor_id: usize) -> Result<(), Error> {
        if let Some(entry) = self.actors.remove(&actor_id) {
            let _actor = entry.1;
            // TODO(shekohex) Remove From Observers
        }
        Ok(())
    }

    /// This method samples the map for elevation problems. If a player is
    /// jumping, this method will sample the map for key elevation changes
    /// and check that the player is not wall jumping. It checks all tiles
    /// in between the player and the jumping destination.
    pub async fn sample_elevation(
        &self,
        distance: u32,
        start: (u32, u32),
        delta: (u32, u32),
        elevation: u16,
    ) -> bool {
        let floor = self.floor.read().await;
        for i in 0..distance {
            let x = start.0 + ((i * delta.0) / distance);
            let y = start.1 + ((i * delta.1) / distance);
            let tile = floor[(x, y)];
            let within_elevation =
                tq_math::within_elevation(tile.elevation, elevation);
            if !within_elevation {
                return false;
            }
        }
        true
    }
}
