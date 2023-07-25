use crate::Error;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use io::{AsyncReadExt, AsyncWriteExt};
use num_enum::FromPrimitive;
use primitives::{Point, Size};
use std::env;
use std::ops::{Index, IndexMut};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io;
use tracing::{debug, trace};
/// This struct encapsulates the coordinate tile grid for a map. It contains
/// methods for loading the map from a flat binary file and for obtaining
/// coordinate values directly from the struct using indexers. The map
/// struct composes from this base struct. If the file does not exist for the
/// map, then a compressed map will be generated from TQ Digital's data map
/// file.
#[derive(Debug, Clone, Default)]
pub struct Floor {
    /// containing access bits for coordinates on the map.
    coordinates: Vec<Tile>,
    /// Size of the map (width and height)
    boundaries: Size<i32>,
    /// true if the map has been loaded correctly.
    loaded: bool,
    /// The path to the map file.
    path: PathBuf,
}

impl Floor {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            coordinates: Default::default(),
            boundaries: Size::default(),
            loaded: false,
            path: path.into(),
        }
    }

    pub fn loaded(&self) -> bool { self.loaded }

    pub fn boundaries(&self) -> &Size<i32> { &self.boundaries }

    pub fn tile(&self, x: u16, y: u16) -> Option<Tile> {
        let i = (x as i32 * self.boundaries.width) + y as i32;
        self.coordinates.get(i as usize).cloned()
    }

    /// This method loads a compressed map from the server's flat file database.
    /// If the file does not exist, the server will make an attempt to find
    /// and convert a dmap version of the map into a compressed map file.
    /// After converting the map, the map will be loaded for the server.
    pub async fn load(&mut self) -> Result<(), Error> {
        if self.loaded {
            return Ok(());
        }
        let data_path = PathBuf::from(env::var("DATA_LOCATION")?);
        let map_path = data_path.join("Maps").join(&self.path);
        trace!("Starting to load map from {}", map_path.display());
        if map_path.exists() {
            let f = File::open(map_path).await?;
            let mut reader = io::BufReader::with_capacity(1024, f);
            let mut buffer = Vec::new();
            reader.read_to_end(&mut buffer).await?;
            let mut buffer = Bytes::from(buffer);
            let width = buffer.get_i32_le();
            let height = buffer.get_i32_le();
            self.boundaries = Size::new(width, height);
            let count = self.boundaries.area() as usize;
            self.coordinates = vec![Tile::default(); count];
            for y in 0..height {
                for x in 0..width {
                    let access = buffer.get_u8().into();
                    let elevation = buffer.get_u16_le();
                    self[(x, y)] = Tile { access, elevation }
                }
            }
            trace!("Loaded Map {}", self.path.display());
        } else {
            trace!("we didn't found the map at {}", map_path.display());
            let mut p = self.path.clone();
            p.set_extension("DMap");
            let orignal_path = data_path
                .join("GameMaps")
                .join("map")
                .join(p)
                .canonicalize()?;
            self.convert(orignal_path).await?;
        }
        self.loaded = true;
        Ok(())
    }

    /// This method unloads the map from memory .. useful when there is no one
    /// on that map. it should get loaded again once needed by calling
    /// [`Self::load`].
    pub fn unload(&mut self) {
        self.boundaries = Size::default();
        self.coordinates.clear();
        self.coordinates = Default::default();
        self.loaded = false;
    }

    /// This method converts a data map from TQ Digital's Conquer Online client
    /// to a compressed map file that only holds access values.
    async fn convert<P: Into<PathBuf>>(
        &mut self,
        path: P,
    ) -> Result<(), Error> {
        let p = path.into();
        trace!("converting {}", p.display());
        let f = File::open(p).await?;
        let mut reader = io::BufReader::with_capacity(1024, f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).await?;
        let mut buffer = Bytes::from(buffer);
        buffer.advance(0x10C);
        let width = buffer.get_i32_le();
        let height = buffer.get_i32_le();
        self.boundaries = Size::new(width, height);
        let count = self.boundaries.area() as usize;
        trace!("Boundaries {:?} with #{} tiles", self.boundaries, count);
        self.coordinates = vec![Tile::default(); count];

        // Get the floor's initial tile information
        for y in 0..height {
            for x in 0..width {
                let mut access = if buffer.get_u16_le() == 0 {
                    TileType::Available
                } else {
                    TileType::Terrain
                };
                let surface = buffer.get_u16_le();
                let elevation = buffer.get_u16_le();
                // Edit the access type and save to the coordinate system:
                if surface == 16 {
                    access = TileType::MarketSpot;
                }
                self[(x, y)] = Tile { access, elevation }
            }
            buffer.advance(4);
        }

        trace!("loaded #{} tiles", count);

        // Get portals from the data map file
        let count = buffer.get_i32_le();
        trace!("start to load #{} portals", count);
        for _ in 0..count {
            let px = buffer.get_i32_le() - 1;
            let py = buffer.get_i32_le() - 1;
            buffer.advance(4);
            for x in 0..3 {
                for y in 0..3 {
                    if py + y < height && px + x < width {
                        self[(px + x, py + y)].access = TileType::Portal;
                    }
                }
            }
        }
        trace!("loaded #{} portals", count);
        // Load scenery data to the map file
        let count = buffer.get_i32_le();
        trace!("start to load #{} scenery data", count);
        for _ in 0..count {
            let ty: SceneryType = (buffer.get_i32_le() as u8).into();
            match ty {
                SceneryType::SceneryObject => {
                    // Get scene data from the DMap
                    let buf = buffer.split_to(260);
                    let scene_file_name = std::str::from_utf8(&buf)?;
                    let (f, _) = scene_file_name.split_at(
                        scene_file_name.find('\0').unwrap_or_default(),
                    );
                    // replace backslashes with forward slashes
                    let scene_file_name =
                        f.replace("map\\", "").replace('\\', "/");
                    let data_path = PathBuf::from(env::var("DATA_LOCATION")?);
                    let scene_path = data_path
                        .join("GameMaps")
                        .join(scene_file_name)
                        .canonicalize()?;
                    trace!("Loading scene file {}", scene_path.display());
                    let px = buffer.get_i32_le();
                    let py = buffer.get_i32_le();
                    let location = Point::new(px, py);
                    // Get scene Data from the scene file
                    let scene = File::open(scene_path).await?;
                    let mut scene_reader =
                        io::BufReader::with_capacity(1024, scene);
                    let mut buffer = Vec::new();
                    scene_reader.read_to_end(&mut buffer).await?;
                    let mut scene_buffer = Bytes::from(buffer);
                    let parts_count = scene_buffer.get_i32_le();
                    trace!("Found #{} parts", parts_count);
                    for _ in 0..parts_count {
                        scene_buffer.advance(0x14C);
                        let scene_width = scene_buffer.get_i32_le();
                        let scene_height = scene_buffer.get_i32_le();
                        let scene_size = Size::new(scene_width, scene_height);
                        trace!("With Size {:?}", scene_size);
                        scene_buffer.advance(4);
                        let sx = scene_buffer.get_i32_le();
                        let sy = scene_buffer.get_i32_le();
                        let start_location = Point::new(sx, sy);
                        scene_buffer.advance(4);
                        // Set the tile information being used by the tile
                        for y in 0..scene_size.height {
                            for x in 0..scene_size.width {
                                let px = location.x + start_location.x - x;
                                let py = location.y + start_location.y - y;
                                let p = Point::new(px, py);
                                let access = if scene_buffer.get_i32_le() == 0 {
                                    TileType::Available
                                } else {
                                    TileType::Terrain
                                };
                                self[(p.x as u32, p.y as u32)].access = access;
                                scene_buffer.advance(8);
                            }
                        }
                    }
                },
                SceneryType::DDSCover => {
                    buffer.advance(0x1A0);
                },
                SceneryType::Effect => {
                    buffer.advance(0x48);
                },
                SceneryType::Sound => {
                    buffer.advance(0x114);
                },
                SceneryType::Unknown => {},
            }
        }
        trace!("loaded #{} scenery data", count);
        self.save().await?;
        Ok(())
    }

    /// This method saves a data map from the client's map folder as a
    /// compressed map for the server. If the file does not exist, the
    /// server will make an attempt save the current map as a compressed map.
    /// Warning: All changes made to the map prior to saving will be final.
    async fn save(&self) -> Result<(), Error> {
        let can_save = !self.path.exists()
            && !self.loaded
            && self.boundaries.area() != 0
            && !self.coordinates.is_empty();
        trace!("Can we save {}? {}", self.path.display(), can_save);
        if can_save {
            let mut buffer = BytesMut::new();
            let width = self.boundaries.width;
            let height = self.boundaries.height;
            buffer.put_i32_le(width);
            buffer.put_i32_le(height);
            for y in 0..height {
                for x in 0..width {
                    let tile = self[(x, y)];
                    buffer.put_u8(tile.access as u8);
                    buffer.put_u16_le(tile.elevation);
                }
            }
            let data_path = PathBuf::from(env::var("DATA_LOCATION")?);
            let map_path = data_path.join("Maps").join(&self.path);
            let f = File::create(map_path).await?;
            let mut writer = io::BufWriter::with_capacity(
                self.boundaries.area() as usize,
                f,
            );
            writer.write_all(&buffer).await?;
            writer.flush().await?;
            debug!("Saved Map {}", self.path.display());
        }
        Ok(())
    }
}

impl Index<(u16, u16)> for Floor {
    type Output = Tile;

    fn index(&self, index: (u16, u16)) -> &Self::Output {
        &self[(index.0 as i32, index.1 as i32)]
    }
}

impl IndexMut<(u16, u16)> for Floor {
    fn index_mut(&mut self, index: (u16, u16)) -> &mut Self::Output {
        &mut self[(index.0 as i32, index.1 as i32)]
    }
}

impl Index<(u32, u32)> for Floor {
    type Output = Tile;

    fn index(&self, index: (u32, u32)) -> &Self::Output {
        &self[(index.0 as i32, index.1 as i32)]
    }
}

impl IndexMut<(u32, u32)> for Floor {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Self::Output {
        &mut self[(index.0 as i32, index.1 as i32)]
    }
}

impl Index<(i32, i32)> for Floor {
    type Output = Tile;

    fn index(&self, index: (i32, i32)) -> &Self::Output {
        let i = (index.0 * self.boundaries.width) + index.1;
        &self.coordinates[i as usize]
    }
}

impl IndexMut<(i32, i32)> for Floor {
    fn index_mut(&mut self, index: (i32, i32)) -> &mut Self::Output {
        let i = (index.0 * self.boundaries.width) + index.1;
        &mut self.coordinates[i as usize]
    }
}

/// This structure encapsulates a tile from the floor's coordinate grid. It
/// contains the tile access information and the elevation of the tile. The
/// map's coordinate grid is composed of these tiles.
#[derive(Debug, Copy, Clone, Default)]
pub struct Tile {
    pub access: TileType,
    pub elevation: u16,
}

/// This enumeration type defines the access types for tiles.
#[derive(Debug, Copy, Clone, FromPrimitive, Eq, PartialEq, Ord, PartialOrd)]
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
