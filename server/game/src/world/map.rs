use super::{Character, Portal};
use crate::systems::{Floor, Tile};
use crate::{constants, Error};
use num_enum::{FromPrimitive, IntoPrimitive};
use parking_lot::RwLock;
use primitives::{Point, Size};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::sync::Arc;
use tq_math::SCREEN_DISTANCE;
use tracing::debug;

type Characters = RwLock<HashMap<u32, Arc<Character>>>;
type Portals = HashSet<Portal>;
type MapRegions = RwLock<Vec<MapRegion>>;

/// This struct encapsulates map information from a compressed map and the
/// database. It includes the identification of the map, pools and methods for
/// character tracking and screen updates, and other methods for processing map
/// actions and events. It composes the floor struct which defines the map's
/// coordinate tile grid.
#[derive(Debug, Default)]
pub struct Map {
    /// The Inner map loaded from the database
    inner: tq_db::map::Map,
    /// where should the player get revived on this map
    #[allow(dead_code)]
    revive_point: Point<u32>,
    /// defines the map's coordinate tile grid.
    floor: Floor,
    /// Holds all Portals in that map.
    portals: Portals,
    /// Holds all MapRegions in that map.
    regions: MapRegions,
}

impl Deref for Map {
    type Target = tq_db::map::Map;

    fn deref(&self) -> &Self::Target { &self.inner }
}

impl Map {
    pub fn new(
        inner: tq_db::map::Map,
        portals: Vec<tq_db::portal::Portal>,
    ) -> Self {
        let portals = portals.into_iter().map(Portal::new).collect();
        Self {
            floor: Floor::new(inner.path.clone()),
            revive_point: Point::new(
                inner.revive_point_x as u32,
                inner.revive_point_y as u32,
            ),
            regions: RwLock::new(Vec::new()),
            portals,
            inner,
        }
    }

    pub fn id(&self) -> u32 { self.inner.map_id as u32 }

    pub fn portals(&self) -> &Portals { &self.portals }

    pub fn tile(&self, x: u16, y: u16) -> Option<Tile> { self.floor.tile(x, y) }

    pub fn with_regions<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&Vec<MapRegion>) -> R,
    {
        f(&self.regions.read())
    }

    #[tracing::instrument(skip(self))]
    pub fn region(&self, x: u16, y: u16) -> Option<MapRegion> {
        let regions = self.regions.read();
        let map_size = self.floor.boundaries();
        let region_size = MapRegion::SIZE;
        let region_x = x as u32 / region_size.width;
        let region_y = y as u32 / region_size.height;
        let width =
            (map_size.width as f32 / region_size.width as f32).ceil() as u32;
        let region_index = region_x * width + region_y;
        tracing::debug!(%x, %y, %region_x, %region_y, %region_index, "Querying Region");
        regions.get(region_index as usize).cloned()
    }

    /// Get a list of the regions that surround the given point.
    pub fn surrunding_regions(&self, x: u16, y: u16) -> Vec<MapRegion> {
        let regions = self.regions.read();
        let map_size = self.floor.boundaries();
        let region_size = MapRegion::SIZE;
        let width =
            (map_size.width as f32 / region_size.width as f32).ceil() as u32;
        let region_x = x as u32 / region_size.width;
        let region_y = y as u32 / region_size.height;
        let region_index = |x, y| x * width + y;
        let mut result = Vec::new();
        for i in 0..constants::WALK_XCOORDS.len() {
            let view_x = region_x as i32 + constants::WALK_XCOORDS[i] as i32;
            let view_y = region_y as i32 + constants::WALK_YCOORDS[i] as i32;
            if view_x.is_negative() || view_y.is_negative() {
                continue;
            }
            let j = region_index(view_x as u32, view_y as u32);
            if let Some(region) = regions.get(j as usize) {
                result.push(region.clone());
            }
        }
        result
    }

    // This method loads a compressed map from the server's flat file database.
    // If the file does not exist, the
    /// server will make an attempt to find and convert a dmap version of the
    /// map into a compressed map file. After converting the map, the map
    /// will be loaded for the server.
    #[tracing::instrument(skip_all, fields(map_id = self.id()))]
    pub async fn load(&self) -> Result<(), Error> {
        debug!("Loading into memory");
        self.floor.load().await?;
        let map_size = self.floor.boundaries();
        let region_size = MapRegion::SIZE;
        // ceil division to get the number of regions
        let height =
            (map_size.height as f32 / region_size.height as f32).ceil() as u32;
        let width =
            (map_size.width as f32 / region_size.width as f32).ceil() as u32;
        let number_of_regions = height * width;
        tracing::trace!(
            %map_size,
            %region_size,
            %height,
            %width,
            number_of_regions,
            "Building regions",
        );
        let mut regions = vec![MapRegion::default(); number_of_regions as usize];
        for y in 0..height {
            for x in 0..width {
                let start_point = Point::new(x, y);
                let region = MapRegion::new(start_point);
                let i = x * width + y;
                regions[i as usize] = region;
                tracing::trace!(%start_point, "Region created");
            }
        }
        assert_eq!(regions.len(), number_of_regions as usize);
        let mut lock = self.regions.write();
        *lock = regions;
        debug!("Map Loaded into memory");
        Ok(())
    }

    #[tracing::instrument(skip_all, fields(map_id = self.id()))]
    pub fn unload(&self) -> Result<(), Error> {
        debug!("Unload from memory");
        self.floor.unload();
        *self.regions.write() = Vec::new();
        debug!("Unloaded from memory");
        Ok(())
    }

    /// This method checks if the map is loaded in memory.
    pub fn loaded(&self) -> bool { self.floor.loaded() }

    /// This method adds the client specified in the parameters to the map pool.
    /// It does this by removing the player from the previous map, then
    /// adding it to the current map. As the character is added, its map,
    /// current tile, and current elevation are changed.
    #[tracing::instrument(skip_all, fields(map_id = self.id(), character_id = me.id()))]
    pub async fn insert_character(
        &self,
        me: Arc<Character>,
    ) -> Result<(), Error> {
        // if the map is not loaded in memory, load it.
        if !self.loaded() {
            self.load().await?;
        }
        // Add the player to the current map
        self.update_region_for(me);
        Ok(())
    }

    /// This method removes the client specified in the parameters from the map.
    /// If the screen of the character still exists, it will remove the
    /// character from each observer's screen.
    #[tracing::instrument(skip(self, character), fields(map_id = self.id(), character_id = character.id()))]
    pub fn remove_character(&self, character: &Character) -> Result<(), Error> {
        // Remove the player from the current map
        let region = self.region(character.x(), character.y());
        if let Some(region) = region {
            region.remove_character(character.id());
        }
        // No One in this map?
        let is_empty = self.with_regions(|r| r.iter().all(|r| r.is_empty()));
        if is_empty {
            // Unload the map from the wrold.
            self.unload()?;
        }
        Ok(())
    }

    /// This method samples the map for elevation problems. If a player is
    /// jumping, this method will sample the map for key elevation changes
    /// and check that the player is not wall jumping. It checks all tiles
    /// in between the player and the jumping destination.
    #[tracing::instrument(skip(self), ret, fields(map_id = self.id()))]
    pub fn sample_elevation(
        &self,
        start: (u16, u16),
        end: (u16, u16),
        elevation: u16,
    ) -> bool {
        let distance = tq_math::get_distance(start, end) as u16;
        // If the distance is 0, we are not moving.
        if distance == 0 {
            return true;
        }
        let delta = tq_math::delta(start, end);
        for i in 0..distance {
            let x = start.0 + ((i.saturating_mul(delta.0)) / distance);
            let y = start.1 + ((i.saturating_mul(delta.1)) / distance);
            let tile = self.floor.tile(x, y);
            match tile {
                Some(tile) => {
                    let within_elevation =
                        tq_math::within_elevation(tile.elevation, elevation);
                    if !within_elevation {
                        return false;
                    }
                },
                None => return false,
            }
        }
        true
    }

    #[tracing::instrument(skip_all, fields(map_id = self.id(), character_id = me.id()))]
    pub fn update_region_for(&self, me: Arc<Character>) {
        let region = self.region(me.x(), me.y());
        let old_region = self.region(me.prev_x(), me.prev_y());
        match (region, old_region) {
            (Some(region), Some(old_region)) if region != old_region => {
                region.insert_character(me.clone());
                old_region.remove_character(me.id());
            },
            (Some(_), Some(_)) => {
                // it is the same region, do nothing
            },
            (Some(region), None) => {
                region.insert_character(me.clone());
            },
            (None, Some(old_region)) => {
                old_region.remove_character(me.id());
            },
            (None, None) => {
                tracing::warn!(
                    x = me.x(),
                    y = me.y(),
                    prev_x = me.prev_x(),
                    prev_y = me.prev_y(),
                    "Can not find a suitable region for character"
                )
            },
        }
    }
}

/// A region or a block is a set of the map which will hold a collection with
/// all entities in an area. This will help us iterating over a limited
/// number of entites when trying to process AI and movement. Instead of
/// iterating a list with thousand entities in the entire map, we'll just
/// iterate the regions around us.
#[derive(Debug, Default, Clone)]
pub struct MapRegion {
    start_point: Point<u32>,
    characters: Arc<Characters>,
}

impl Eq for MapRegion {}
impl PartialEq for MapRegion {
    fn eq(&self, other: &Self) -> bool { self.start_point == other.start_point }
}

impl MapRegion {
    /// WIDTH and HEIGHT are the number of tiles in a region.
    pub const SIZE: Size<u32> =
        Size::new(SCREEN_DISTANCE as _, SCREEN_DISTANCE as _);

    pub fn new(start_point: Point<u32>) -> Self {
        Self {
            start_point,
            characters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn is_empty(&self) -> bool { self.with_characters(|c| c.is_empty()) }

    pub fn try_character(&self, id: u32) -> Option<Arc<Character>> {
        self.with_characters(|c| c.get(&id).cloned())
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

    pub fn insert_character(&self, character: Arc<Character>) {
        self.with_characters_mut(|c| {
            c.insert(character.id(), character);
        });
    }

    pub fn remove_character(&self, id: u32) -> Option<Arc<Character>> {
        self.with_characters_mut(|c| c.remove(&id))
    }
}

#[derive(Debug, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
#[allow(unused)]
pub enum Maps {
    #[num_enum(default)]
    Unknwon = 0,
    Desert = 1000,
    Newplain = 1002,
    Mine01 = 1003,
    Forum = 1004,
    Arena = 1005,
    Horse = 1006,
    Star01 = 1100,
    Star02 = 1101,
    Star03 = 1102,
    Star04 = 1103,
    Star05 = 1104,
    Star10 = 1105,
    Star06 = 1106,
    Star07 = 1107,
    Star08 = 1108,
    Star09 = 1109,
    Smith = 1007,
    Grocery = 1008,
    Newbie = 1010,
    Woods = 1011,
    Sky = 1012,
    Tiger = 1013,
    Dragon = 1014,
    Island = 1015,
    Qiling = 1016,
    Canyon = 1020,
    Mine = 1021,
    Brave = 1022,
    MineOne = 1025,
    MineTwo = 1026,
    MineThree = 1027,
    MineFour = 1028,
    MineOne2 = 1029,
    MineTwo2 = 1030,
    MineThree2 = 1031,
    MineFour2 = 1032,
    Prison = 6000,
    Street = 1036,
    FactionBlack = 1037,
    Faction = 1038,
    Playground = 1039,
    Skycut = 1040,
    Skymaze = 1041,
    LineupPass = 1042,
    Lineup = 1043,
    Riskisland = 1051,
    Skymaze1 = 1060,
    Skymaze2 = 1061,
    Skymaze3 = 1062,
    Star = 1064,
    Boa = 1070,
    PArena = 1080,
    Newcanyon = 1075,
    Newwoods = 1076,
    Newdesert = 1077,
    Newisland = 1078,
    MysIsland = 1079,
    IdlandMap = 1082,
    ParenaM = 1090,
    ParenaS = 1091,
    House01 = 1098,
    House03 = 1099,
    Sanctuary = 1601,
    Task01 = 1201,
    Task02 = 1202,
    Task04 = 1204,
    Task05 = 1205,
    Task07 = 1207,
    Task08 = 1208,
    Task10 = 1210,
    Task11 = 1211,
    IslandSnail = 1212,
    DesertSnail = 1213,
    CanyonFairy = 1214,
    WoodsFairy = 1215,
    NewplainFairy = 1216,
    MineA = 1500,
    MineB = 1501,
    MineC = 1502,
    MineD = 1503,
    STask01 = 1351,
    STask02 = 1352,
    STask03 = 1353,
    STask04 = 1354,
    Slpk = 1505,
    Hhpk = 1506,
    Blpk = 1507,
    Ympk = 1508,
    Mfpk = 1509,
    Faction01 = 1550,
    Jokul01 = 1615,
    Tiemfiles = 1616,
    Dgate = 2021,
    Dsquare = 2022,
    Dcloister = 2023,
    Dsigil = 2024,
    Cordiform = 1645,
    Nhouse04 = 601,
    ArenaNone = 700,
    Fairylandpk07 = 1760,
    Halloween2007A = 1766,
    Halloween2007Boss = 1767,
    WoodsZ = 1066,
    QilingZ = 1067,
    Fairylandpk03 = 1764,
    IcecryptLev1 = 1762,
}

#[cfg(test)]
mod tests {
    use futures::FutureExt;

    use super::*;
    use crate::test_utils::*;

    #[tokio::test]
    async fn map_regions() -> Result<(), Error> {
        with_test_env(tracing::Level::TRACE, |state, _actors| {
            async move {
                let test_map_id = Maps::Arena;
                let map = state.try_map(test_map_id.into())?;
                map.load().await?;
                let my_region = map.region(50, 50);
                assert!(my_region.is_some(), "Can't find a region on (50, 50)");
                let my_region = map.region(67, 50);
                assert!(my_region.is_some(), "Can't find a region on (67, 50)");
                Ok(())
            }
            .boxed()
        })
        .await
    }
}
