use num_enum::FromPrimitive;
/// This structure encapsulates a tile from the floor's coordinate grid. It
/// contains the tile access information and the elevation of the tile. The
/// map's coordinate grid is composed of these tiles.
#[derive(Debug, Copy, Clone, Default)]
pub struct Tile {
    pub access: TileType,
    pub elevation: u16,
}

/// This enumeration type defines the access types for tiles.
#[derive(Debug, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum TileType {
    Terrain = 0,
    Npc = 1,
    Monster = 2,
    Portal = 3,
    Item = 4,
    MarketSpot = 5,
    Available = 6,
    #[num_enum(default)]
    Unknown = u8::MAX,
}

impl Default for TileType {
    fn default() -> Self { Self::Unknown }
}

/// This enumeration type defines the types of scenery files used by the client.
#[derive(Debug, Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum SceneryType {
    SceneryObject = 1,
    DDSCover = 4,
    Effect = 10,
    Sound = 15,
    #[num_enum(default)]
    Unknown = u8::MAX,
}
