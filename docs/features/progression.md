# Progression (XP, Levels, Attributes, Rebirth) — ❌ Missing

## Current state

The data model is ready but no mechanics exist. The `characters` table
([1_characters.sql](../../migrations/1_characters.sql)) persists level, experience,
attribute points, str/agi/vit/spi, HP/MP, class, previous class, and rebirths,
and [entities/character.rs](../../server/game/src/entities/character.rs) exposes all
of them — but nothing ever changes them after creation.

Unhandled pieces:

- `ActionType::LevelUp` (92) and `Reborn` (94) — enumerated in
  [msg_action.rs](../../server/game/src/packets/msg_action.rs#L28), no handlers.
- No `MsgUserAttrib` packet (the 5017 packet for pushing stat changes — HP,
  level, XP bar — to the client and observers).
- No attribute-point allocation handler (client sends `MsgUserAttrib`-adjacent
  requests when spending points).
- No XP sources (no combat, no quests) and no level-up stat tables.
- Mining (`ActionType::Mine`, 99) — the classic alternate progression loop —
  is unhandled.
- `entities/basic.rs` has TODOs for max-HP handling and more status flags
  ([basic.rs:214](../../server/game/src/entities/basic.rs#L214)).

## What's needed

1. **`MsgUserAttrib`** — prerequisite for showing any stat change.
2. **XP curve data** + level-up: auto stats below 120, attribute points after,
   HP/MP recalculation, screen broadcast of the level-up effect.
3. **Attribute allocation** — spend `attribute_points` with validation.
4. **Rebirth** — class-change rules, stat reset, previous-class bonuses.
5. **Mining** — tick-based resource gain in mines; needs items for ore.

Blocked primarily by combat/monsters (XP sources) and items (HP pots, gear).
