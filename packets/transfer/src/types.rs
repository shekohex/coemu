#[cfg(not(feature = "std"))]
use alloc::string::String;

pub struct Realm {
    pub id: u32,
    pub name: String,
    pub game_ip_address: String,
    pub game_port: u16,
}
