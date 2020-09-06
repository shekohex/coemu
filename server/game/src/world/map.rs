use super::Character;
use crate::{
    db,
    entities::BaseEntity,
    systems::{Floor, Tile},
    Error,
};
use dashmap::DashMap;
use primitives::Point;
use std::{ops::Deref, sync::Arc};
use tokio::sync::RwLock;
use tracing::debug;

type Characters = Arc<DashMap<u32, Character>>;

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
    characters: Characters,
    /// where should the player get revived on this map
    revive_point: Arc<Point<u32>>,
    /// defines the map's coordinate tile grid.
    floor: Arc<RwLock<Floor>>,
}

impl Deref for Map {
    type Target = db::Map;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl Map {
    pub fn new(inner: db::Map) -> Self {
        Self {
            floor: Arc::new(RwLock::new(Floor::new(inner.path.clone()))),
            characters: Arc::new(DashMap::new()),
            revive_point: Arc::new(Point::new(
                inner.revive_point_x as u32,
                inner.revive_point_y as u32,
            )),
            inner: Arc::new(inner),
        }
    }

    pub fn id(&self) -> u32 { self.inner.map_id as u32 }

    pub fn characters(&self) -> &Characters { &self.characters }

    pub async fn tile(&self, x: u16, y: u16) -> Result<Tile, Error> {
        let floor = self.floor.read().await;
        Ok(floor[(x as u32, y as u32)])
    }

    // This method loads a compressed map from the server's flat file database.
    // If the file does not exist, the
    /// server will make an attempt to find and convert a dmap version of the
    /// map into a compressed map file. After converting the map, the map
    /// will be loaded for the server.
    pub async fn load(&self) -> Result<(), Error> {
        debug!("Loading {} into memory", self.id());
        self.floor.write().await.load().await?;
        debug!("Map {} Loaded into memory", self.id());
        Ok(())
    }

    pub async fn unload(&self) -> Result<(), Error> {
        debug!("Unload {} from memory", self.id());
        self.floor.write().await.unload();
        debug!("Map {} Unloaded from memory", self.id());
        Ok(())
    }

    pub async fn loaded(&self) -> bool { self.floor.read().await.loaded() }

    /// This method adds the client specified in the parameters to the map pool.
    /// It does this by removing the player from the previous map, then
    /// adding it to the current map. As the character is added, its map,
    /// current tile, and current elevation are changed.
    pub async fn insert_character(
        &self,
        character: Character,
    ) -> Result<(), Error> {
        // if the map is not loaded in memory, load it.
        if !self.loaded().await {
            self.load().await?;
        }
        // Remove the client from the previous map
        character
            .owner()
            .map()
            .await?
            .remove_character(character.id())
            .await?;
        // Add the player to the current map
        let added = self
            .characters
            .insert(character.id(), character.clone())
            .is_none();
        character.owner().set_map(self.clone()).await?;
        if added {
            let floor = self.floor.read().await;
            let tile = floor[(character.x() as u32, character.y() as u32)];
            character.set_elevation(tile.elevation);
            // TODO(shekohex): Send environment packets.
        }
        Ok(())
    }

    /// This method removes the client specified in the parameters from the map.
    /// If the screen of the character still exists, it will remove the
    /// character from each observer's screen.
    pub async fn remove_character(&self, id: u32) -> Result<(), Error> {
        if let Some(entry) = self.characters.remove(&id) {
            let character = entry.1;
            let screen = character.owner().screen().await?;
            screen.remove_from_observers().await?;
        }
        // No One in this map?
        if self.characters.is_empty() {
            // Unload the map from the wrold.
            self.unload().await?;
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
