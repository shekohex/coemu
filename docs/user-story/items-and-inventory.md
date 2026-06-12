# Epic: Items & Inventory

> Source: [items-and-inventory.md](../features/items-and-inventory.md) (❌ missing),
> [MsgItem/MsgItemInfo/MsgMapItem notes](../notes/co-5017-protocol-notes.md).
> This epic unblocks combat, economy, trade, and drops.

## ITM-1: Item type catalogue

**As an** operator **I want** the 5017 item catalogue (`itemtype.dat`)
ingested into the database **so that** the server knows every item's stats,
requirements, and price without hardcoding.

**Acceptance criteria:**
- A migration + seeding pipeline (mirroring `generated_maps`/`generated_npcs`)
  loads item types: id, name, class/level/sex/stat requirements, price,
  attack/defense/agility/HP/MP bonuses, durability, gem sockets, plus value.
- The decryption (TQ file cipher, seed 9527) and `@@`-field parsing follow the
  [wiki itemtype.dat format](../notes/co-5017-protocol-notes.md#game-data-files);
  field count validated per record, bad rows rejected with a report.
- Item types load into an in-memory read-only catalogue at boot (same pattern
  as maps), not queried per-action.

## ITM-2: Persistent owned items

**As a** player **I want** my items stored with my character **so that** they
survive logout and server restarts.

**Acceptance criteria:**
- `items` table: unique item id, owner character, item type, position
  (inventory / equipment slot 1–8 / storage 201+ / ground), durability +
  max, gem sockets 1–2, plus (Magic3), bless, enchant, amount.
- Positions use the 5017 enumeration (0 inventory, 1 helmet … 8 boots,
  201 storage, 254 ground, 255 none) from
  [MsgItemInfo notes](../notes/co-5017-protocol-notes.md#msgiteminfo-1008--item-description-5017-layout).
- ⚠ Item ids are not trivially sequential/predictable (LFSR or equivalent),
  per the wiki identifier warning.
- Item mutations persist via transactions when value moves between owners,
  write-behind otherwise.

## ITM-3: See my inventory on login

**As a** player **I want** my inventory and equipment to appear when I enter
the world **so that** I can play with the items I own.

**Acceptance criteria:**
- `ActionType::SendItems` (currently an echo stub at
  [msg_action.rs:354](../../server/game/src/packets/msg_action.rs#L354)) responds
  with one `MsgItemInfo` (1008, 36-byte 5017 layout, action=ADD_ITEM) per item.
- Equipped items appear in their slots; other players receive my equipment
  via the existing spawn flow (action=OTHER_PLAYER_EQUIPMENT on query).
- Inventory capacity (40) is enforced on every acquisition path.

## ITM-4: Equip and unequip gear

**As a** player **I want** to equip weapons and armor **so that** my stats
and appearance reflect my gear.

**Acceptance criteria:**
- `MsgItem` (1009) EQUIP (4) moves a validated item to the requested slot;
  UNEQUIP/UPDATE (5/6) pushes the change back; swapped items return to
  inventory.
- Server revalidates class, level, sex, and str/agi/vit/spi requirements from
  the catalogue — client checks are advisory only.
- Equipment changes recompute derived stats (attack, defense, dodge, HP/MP
  bonuses) and broadcast the new look to the screen.
- Dual-wield/shield rules for weapon-L (slot 5) enforced.

## ITM-5: Use consumables

**As a** player **I want** to use potions and scrolls **so that** I can
recover HP/MP and teleport.

**Acceptance criteria:**
- Using a consumable decrements amount (stack) or deletes the item, applies
  the effect (HP/MP restore via `MsgUserAttrib`), and respects a server-side
  use cooldown.
- Effects resolve from the catalogue's action id — no per-item hardcoding.

## ITM-6: Drop and pick up items

**As a** player **I want** to drop items/money and pick up floor loot
**so that** I can manage inventory and collect monster drops.

**Acceptance criteria:**
- DROP (3) / DROP_MONEY (12) create floor items via `MsgMapItem` (1101)
  CREATE with ids from the map-item range; PICK (3) requests validate
  same-cell proximity and inventory space.
- Floor items expire on a timer (DELETE broadcast); drops from MON-5 use
  short-term owner protection before anyone can loot.
- Dropped items are removed from the dropper atomically before the floor
  item is visible — no duplication window on disconnect.
- Cursed/quest items (monopoly flag in the catalogue) refuse dropping.

## ITM-7: Repair and durability

**As a** player **I want** equipment to wear and be repairable **so that**
item maintenance is part of the economy.

**Acceptance criteria:**
- Durability decrements on combat events; DURABILITY (17) updates push at
  threshold crossings, not every hit.
- REPAIR (14) at a shop NPC charges the 5017 repair cost from current/max
  durability and item price; broken (0-durability) gear stops applying stats.

## ITM-8: Upgrade items (composition)

**As a** player **I want** to improve item quality/level/plus with
dragonballs, meteors, and gems **so that** gear progression exists (the
5017 WuXing Oven loop).

**Acceptance criteria:**
- IMPROVE (19) / UPLEV (20) / COMBINE (8) / ENCHANT (28, new in 5017)
  validate and consume the correct fuel item, then update the target via
  `MsgItemInfo` UPDATE.
- ⚠ Server checks fuel item type matches the action and **differs from the
  target item**, and that both belong to the requester — the wiki's explicit
  anti-dupe warning.
- ⚠ Proximity validated: no upgrading from a distance or another map.
- Composition rates are data-driven (wiki page is a stub — rates sourced from
  COPSv6/Soul and kept in one tunable table).
