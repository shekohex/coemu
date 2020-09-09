use super::Character;
use crate::{
    db,
    entities::BaseEntity,
    systems::{Floor, Tile},
    Error,
};
use primitives::Point;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::RwLock;
use tracing::debug;

type Characters = Arc<RwLock<HashMap<u32, Character>>>;

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
            characters: Arc::new(RwLock::new(HashMap::new())),
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
        {
            // Remove the client from the previous map
            character
                .owner()
                .map()
                .await?
                .remove_character(character.id())
                .await?;
        }
        // Add the player to the current map
        let added = self
            .characters
            .write()
            .await
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
        if let Some(character) = self.characters.write().await.remove(&id) {
            let screen = character.owner().screen().await?;
            screen.remove_from_observers().await?;
        }
        // No One in this map?
        if self.characters.read().await.is_empty() {
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
        start: (u16, u16),
        end: (u16, u16),
        elevation: u16,
    ) -> bool {
        let distance = tq_math::get_distance(start, end) as u16;
        let delta = tq_math::delta(start, end);
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

#[derive(Debug)]
#[repr(u16)]
#[allow(unused)]
pub enum Maps {
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
