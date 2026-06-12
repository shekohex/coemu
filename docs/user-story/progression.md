# Epic: Progression

> Source: [progression.md](../features/progression.md) (❌ missing — data
> model exists, mechanics don't),
> [MsgUserAttrib/MsgAllot/levelexp notes](../notes/co-5017-protocol-notes.md).

## PRG-1: Stat changes reach the client

**As a** player **I want** my HP/MP/XP/level/status changes shown instantly
**so that** the UI reflects reality.

**Acceptance criteria:**
- `MsgUserAttrib` (1017) implemented with the 5017 attribute enumeration
  (incl. EMONEY 30, LUCKY_SECONDS 29, XP 31) and multi-attribute batching.
- Self-only attributes (money, exp) go to the owner; visible ones (life,
  level, USERSTATUS) also broadcast to the screen.
- This story is a prerequisite for combat, items, and economy — deliver first.

## PRG-2: Gain XP and level up

**As a** player **I want** XP from kills to level me up **so that** my
character grows.

**Acceptance criteria:**
- XP curve seeded from `levelexp.dat` (flat u32 array, XOR key documented in
  the [protocol notes](../notes/co-5017-protocol-notes.md#game-data-files)).
- `ActionType::LevelUp` (92) flow: at threshold, level increments, HP/MP
  recalc, level-up effect broadcast, `MsgUserAttrib` LEV/EXP/MAXLIFE updates.
- Below level 120 stats auto-allocate per class table; after that levels
  grant free attribute points.
- XP sources: monster kills (with level-gap scaling and team sharing), later
  quests/mining.

## PRG-3: Spend attribute points

**As a** player **I want** to allocate attribute points **so that** I can
build my character.

**Acceptance criteria:**
- `MsgAllot` (1024: force/speed/health/soul bytes) handler applies points
  and recomputes derived stats.
- ⚠ Server validates the sum against the character's unallocated points
  before applying — the wiki explicitly warns the client check is spoofable.
- Persisted; ADDPOINT attribute update confirms the new balance.

## PRG-4: Rebirth

**As a** player **I want** to be reborn at 110+ **so that** late-game
progression continues (5017 supports first rebirth).

**Acceptance criteria:**
- `Reborn` (94) validates level/class requirements and water-of-life item;
  resets level/stats per 5017 rules, stores previous class (column exists),
  grants rebirth bonuses, sets METEMPSYCHOSIS attribute and rebirth item
  effect (Magic1 on equipment).

## PRG-5: Mining

**As a** player **I want** to mine ore in mines **so that** an AFK-adjacent
progression/economy loop exists.

**Acceptance criteria:**
- `ActionType::Mine` (99) starts a mining loop on mine-flagged tiles with a
  pickaxe equipped; ticks roll ore/gem drops into inventory (ITM dependency)
  until movement/combat interrupts.
