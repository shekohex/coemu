use crate::entities::Entity;
use crate::packets::MsgNpcInfo;
use crate::Error;
use num_enum::{FromPrimitive, IntoPrimitive};
use tq_network::ActorHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum NpcKind {
    #[default]
    None = 0,
    ShopKeeper = 1,
    Task = 2,
    Storage = 3,
    Trunck = 4,
    Face = 5,
    Forge = 6,
    Embed = 7,

    Statuary = 9,
    SynFlag = 10,

    Booth = 14,
    SynTrans = 15,
    BoothFlag = 16,

    Dice = 19, // Game

    WeaponGoal = 21,
    MagicGoal = 22,
    BowGoal = 23,
    Target = 24,
    Furniture = 25,
    CityGate = 26,
    NeighbourGate = 27,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum NpcSort {
    #[default]
    None = 0,
    Task = 1 << 0,
    Recycle = 1 << 1,
    Scene = 1 << 2,
    LinkMap = 1 << 3,
    DieAction = 1 << 4,
    DelEnable = 1 << 5,
    Event = 1 << 6,
    Table = 1 << 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum NpcBase {
    #[default]
    None = 0,
    Lamp = 1,
    LowShelf = 2,
    Cabinet = 3,
    HighShelf = 4,
    BombeChest = 5,
    RosewoodCabinet = 6,
    HighCabinet = 7,
    FoldingScreen = 8,
    Dresser = 9,
    BasinRack = 10,
    Chair = 11,
    EndTable = 12,
    LeftGate = 24,
    RightGate = 27,
}

#[derive(Debug)]
pub struct Npc {
    inner: tq_db::npc::Npc,
    entity: Entity,
    kind: NpcKind,
    sort: NpcSort,
    base: NpcBase,
}

impl Npc {
    pub fn new(inner: tq_db::npc::Npc) -> Self {
        let entity = Entity::from(&inner);
        let kind = NpcKind::from(inner.kind as u8);
        let sort = NpcSort::from(inner.sort as u8);
        let base = NpcBase::from(inner.base as u8);
        Self {
            entity,
            inner,
            kind,
            sort,
            base,
        }
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.inner.id as _
    }

    pub fn kind(&self) -> NpcKind {
        self.kind
    }

    pub fn sort(&self) -> NpcSort {
        self.sort
    }

    pub fn base(&self) -> NpcBase {
        self.base
    }

    #[inline]
    pub fn entity(&self) -> &Entity {
        &self.entity
    }

    pub fn is_shopkeeper(&self) -> bool {
        self.kind == NpcKind::ShopKeeper
    }

    pub fn is_storage(&self) -> bool {
        self.kind == NpcKind::Storage
    }

    pub fn is_booth(&self) -> bool {
        self.kind == NpcKind::Booth
    }

    #[tracing::instrument(skip(self, to), fields(npc = self.entity.id()))]
    pub(super) async fn send_spawn(&self, to: &ActorHandle) -> Result<(), Error> {
        let msg = if self.is_booth() {
            MsgNpcInfo::from_npc_with_name(self)
        } else {
            MsgNpcInfo::new(self)
        };
        to.send(msg).await?;
        tracing::trace!("sent spawn");
        Ok(())
    }
}

impl From<tq_db::npc::Npc> for Npc {
    fn from(value: tq_db::npc::Npc) -> Self {
        Self::new(value)
    }
}
