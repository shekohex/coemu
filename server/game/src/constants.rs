#![allow(unused)]

pub const SYSTEM: &str = "SYSTEM";
pub const ALL_USERS: &str = "ALLUSERS";
pub const ANSWER_OK: &str = "ANSWER_OK";
pub const NEW_ROLE: &str = "NEW_ROLE";

pub const MAX_TXT_LEN: usize = 250;

pub const HAIR_STYLES: [i16; 12] = [10, 11, 13, 14, 15, 24, 30, 35, 37, 38, 39, 40];

pub const WALK_XCOORDS: [i8; 8] = [0, -1, -1, -1, 0, 1, 1, 1];
pub const WALK_YCOORDS: [i8; 8] = [1, 1, 0, -1, -1, -1, 0, 1];

pub const NPC_ID_MIN: u32 = 1;
pub const DYN_NPC_ID_MIN: u32 = 100001;
pub const DYN_NPC_ID_MAX: u32 = 199999;
pub const MONSTER_ID_MIN: u32 = 400001;
pub const MONSTER_ID_MAX: u32 = 499999;
pub const PET_ID_MIN: u32 = 500001;
pub const PET_ID_MAX: u32 = 599999;
pub const NPC_ID_MAX: u32 = 700000;
pub const CALL_PET_ID_MIN: u32 = 700001;
pub const CALL_PET_ID_MAX: u32 = 799999;
pub const CHARACTER_ID_MIN: u32 = 1000000;
pub const CHARACTER_ID_MAX: u32 = 10000000;

pub const fn is_npc(id: u32) -> bool {
    id >= NPC_ID_MIN && id <= NPC_ID_MAX
}

pub const fn is_terrain_npc(id: u32) -> bool {
    id >= DYN_NPC_ID_MIN && id <= DYN_NPC_ID_MAX
}

pub const fn is_monster(id: u32) -> bool {
    id >= MONSTER_ID_MIN && id <= MONSTER_ID_MAX
}

pub const fn is_pet(id: u32) -> bool {
    id >= PET_ID_MIN && id <= PET_ID_MAX
}

pub const fn is_call_pet(id: u32) -> bool {
    id >= CALL_PET_ID_MIN && id <= CALL_PET_ID_MAX
}

pub const fn is_character(id: u32) -> bool {
    id >= CHARACTER_ID_MIN && id <= CHARACTER_ID_MAX
}
