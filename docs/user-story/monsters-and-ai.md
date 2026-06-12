# Epic: Monsters & AI

> Source: [monsters-and-ai.md](../features/monsters-and-ai.md) (❌ missing),
> [identifier ranges](../notes/co-5017-protocol-notes.md#entity-identifier-ranges).
> Note `magictype.dat`/`monstertype.dat` formats are wiki stubs — stats will
> be sourced from COPSv6/Soul-derived datasets.

## MON-1: Monster catalogue and spawn data

**As an** operator **I want** monster types and spawn areas in the database
**so that** maps populate without code changes.

**Acceptance criteria:**
- `monstertype` table (type id, name, look, level, HP, min/max attack,
  defense, dodge, attack speed/range, view range, move speed, drop money
  range, drop item rules) and `spawns` table (map, region rect, type, max
  count, respawn seconds), seeded like `generated_npcs`.
- Loaded into memory at boot alongside the map/NPC data.

## MON-2: Monsters appear in the world

**As a** player **I want** to see monsters on the map **so that** the world
feels alive.

**Acceptance criteria:**
- A `Monster` entity variant resolves the `send_spawn` `todo!` at
  [entities/mod.rs:65](../../server/game/src/entities/mod.rs#L65) — spawning a
  monster into a screen must not panic.
- Monster role IDs allocated from 400,001–499,999 with per-spawn-area
  recycling, per the identifier notes.
- Spawn generators keep each area stocked to its max count and respawn dead
  monsters after the configured delay.
- Monsters integrate with the existing screen system: appear/disappear on
  player movement exactly like players do today.

## MON-3: Monsters behave

**As a** player **I want** monsters to wander, aggro, chase, and attack
**so that** hunting has risk.

**Acceptance criteria:**
- A server tick loop (does not exist yet) drives an AI state machine:
  idle/wander → aggro (passive types only when attacked; aggressive types on
  proximity) → chase → attack → leash back when dragged too far.
- AI runs only in regions that contain players (anchor on the region grid in
  [map.rs](../../server/game/src/world/map.rs)) so 5k-CCU tick cost stays bounded.
- Pathing respects DMap tile access; attacks respect range and attack speed.

## MON-4: Killing monsters rewards XP

**As a** player **I want** XP when my team or I kill a monster **so that**
hunting progresses my character.

**Acceptance criteria:**
- On monster death: death animation/state, `MsgInteract` KILL confirmation
  (type 14) to the killer, XP awarded per the level-gap rules, corpse
  despawn timer.
- Team XP sharing rules apply when the killer is in a team (links SOC-4).

## MON-5: Monsters drop loot

**As a** player **I want** monsters to drop money and items **so that**
farming is rewarding.

**Acceptance criteria:**
- Drop rolls from the monster's drop rules produce floor items/money via the
  ITM-6 flow at the corpse location.
- Drops are owner-protected for a window (killer/team only) before becoming
  free-for-all, then expire.
- Drop rates are data-driven and tunable per type.
