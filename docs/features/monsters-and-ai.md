# Monsters & AI — ❌ Missing

## Current state

Nothing exists: no monster tables in [migrations/](../../migrations/), no spawn
generators, no AI tick loop, no respawn timers. The only non-player entities
are stationary NPCs.

Two code paths actively block monsters today:

- `GameEntity::send_spawn` is `todo!("send_spawn for non-character entities")`
  ([entities/mod.rs:65](../../server/game/src/entities/mod.rs#L65)) — adding a
  monster to a screen would panic the handler.
- `TalkChannel::Monster` exists in the chat enum but is unused.

## What's needed

1. **Data** — `monstertype` (stats, look, drops) and `spawns` (map, region,
   count, rate) tables seeded from 5017 client/data sources, with `tq-db`
   models alongside [crates/db/src/npc.rs](../../crates/db/src/npc.rs).
2. **Monster entity** — a `Monster` variant in
   [entities/](../../server/game/src/entities/mod.rs) with HP/state, plus a
   `MsgPlayer`-equivalent spawn payload (monsters use the same spawn packet
   with mesh/monster-type fields in 5017) to resolve the `todo!`.
3. **Spawn system** — per-map generators that keep regions stocked and respawn
   on a timer; the region grid in
   [world/map.rs](../../server/game/src/world/map.rs) is a good anchor for
   activating AI only in regions containing players.
4. **AI loop** — a server tick (none exists yet) driving wander, aggro,
   chase, attack, leash, and corpse/despawn states.
5. **Integration** — combat ([combat-and-pk.md](../features/combat-and-pk.md))
   for damage/death, items for drop tables, progression for XP rewards.
