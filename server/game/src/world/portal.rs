use crate::utils::LoHi;
use std::hash::Hash;
use std::ops::Deref;

#[derive(Debug)]
pub struct Portal {
    inner: tq_db::portal::Portal,
}

impl Deref for Portal {
    type Target = tq_db::portal::Portal;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Portal {
    pub fn new(inner: tq_db::portal::Portal) -> Self {
        Self { inner }
    }

    pub fn uid(&self) -> u32 {
        self.inner.id as u32
    }

    pub fn id(&self) -> u32 {
        u32::constract(self.from_y(), self.from_x())
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_map_id(&self) -> u32 {
        self.inner.from_map_id as u32
    }

    pub fn to_map_id(&self) -> u32 {
        self.inner.to_map_id as u32
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_x(&self) -> u16 {
        self.inner.from_x as u16
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_y(&self) -> u16 {
        self.inner.from_y as u16
    }

    pub fn to_x(&self) -> u16 {
        self.inner.to_x as u16
    }

    pub fn to_y(&self) -> u16 {
        self.inner.to_y as u16
    }
}

impl PartialEq for Portal {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Portal {}

impl Hash for Portal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.id());
    }
}
