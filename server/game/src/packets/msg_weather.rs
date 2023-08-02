use num_enum::{FromPrimitive, IntoPrimitive};
use rand::Rng;
use serde::Serialize;
use tq_network::PacketID;

/// These enumeration type values are hard-coded into the client and server,
/// sent when the [`MsgWeather`] packet.
#[derive(Debug, Clone, Copy, FromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum WeatherKind {
    #[num_enum(default)]
    Unknwon = 0,
    None = 1,
    Rain = 2,
    Snow = 3,
    RainWind = 4,
    AutumnLeaves = 5,
    CherryBlossomPetals = 7,
    CherryBlossomPetalsWind = 8,
    BlowingCotten = 9,
    Atoms = 10,
}

/// This packet is sent from the game server to the client for invoking weather
/// on a map. This packet must be sent when the player changes maps; otherwise,
/// the weather will be cleared by the client.
#[derive(Debug, Serialize, Clone, PacketID)]
#[packet(id = 1016)]
pub struct MsgWeather {
    /// Weather type
    kind: u32,
    /// Range: 0 - 999
    intensity: u32,
    /// Range: 0 - 359
    direction: u32,
    /// Color in ARGB (Default: 0x00FFFFFF)
    color: u32,
}

impl MsgWeather {
    pub fn new(kind: WeatherKind) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            kind: kind.into(),
            intensity: rng.gen_range(1..=999),
            direction: rng.gen_range(1..=359),
            color: 0x00FFFFFF,
        }
    }

    pub fn none() -> Self { Self::new(WeatherKind::None) }

    pub fn rain() -> Self { Self::new(WeatherKind::Rain) }

    pub fn snow() -> Self { Self::new(WeatherKind::Snow) }

    pub fn rain_wind() -> Self { Self::new(WeatherKind::RainWind) }

    pub fn autumn_leaves() -> Self { Self::new(WeatherKind::AutumnLeaves) }

    pub fn cherry_blossom_petals() -> Self {
        Self::new(WeatherKind::CherryBlossomPetals)
    }

    pub fn cherry_blossom_petals_wind() -> Self {
        Self::new(WeatherKind::CherryBlossomPetalsWind)
    }

    pub fn blowing_cotten() -> Self { Self::new(WeatherKind::BlowingCotten) }

    pub fn atoms() -> Self { Self::new(WeatherKind::Atoms) }
}
