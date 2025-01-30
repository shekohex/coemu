pub trait FloorItem: Default {
    fn money(&self) -> u32;
    fn map_id(&self) -> u32;
    fn x(&self) -> u16;
    fn y(&self) -> u16;
}

#[derive(Debug, Clone, Default)]
pub struct Item;

impl FloorItem for Item {
    fn money(&self) -> u32 {
        0
    }

    fn map_id(&self) -> u32 {
        0
    }

    fn x(&self) -> u16 {
        0
    }

    fn y(&self) -> u16 {
        0
    }
}
