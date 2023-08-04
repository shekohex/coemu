use crate::entities::Entity;

#[derive(Debug)]
pub struct Npc {
    entity: Entity,
}

impl Npc {
    #[inline]
    pub fn id(&self) -> u32 { self.entity.id() }

    #[inline]
    pub fn entity(&self) -> &Entity { &self.entity }
}
