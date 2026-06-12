# World, Maps, Portals & Weather — ✅ Implemented

## Maps

- Loaded at startup from SQLite (`maps` table,
  [crates/db/src/map.rs](../../crates/db/src/map.rs); seed data in
  [8_generated_maps.sql](../../migrations/8_generated_maps.sql) generated from
  [data/Maps/Maps.csv](../../data/Maps/Maps.csv)).
- Runtime `Map` ([world/map.rs](../../server/game/src/world/map.rs)) owns its
  portals, NPCs, weather, ARGB color, and a region grid of entities.
- Tile data (walkability, elevation) comes from the client's DMap files via
  the `Floor` system ([systems/floor.rs](../../server/game/src/systems/floor.rs)),
  loaded lazily per map; the
  [gamemap-decoder](../../tools/gamemap-decoder/src/main.rs) tool decodes
  `GameMap.dat`. Fixture maps are fetched by
  [scripts/fetch-test-fixtures.bash](../../scripts/fetch-test-fixtures.bash).
- `MsgMapInfo` ([msg_map_info.rs](../../server/game/src/packets/msg_map_info.rs))
  sends map id/flags; `ActionType::MapARGB` sends the map color.

## Regions

Maps are partitioned into regions; entities are tracked per region with weak
references, movement updates re-home entities
(`Map::update_region_for`), and packets can be broadcast per region (used by
chat and movement fan-out).

## Portals

- `portals` table ([crates/db/src/portal.rs](../../crates/db/src/portal.rs), seeded
  from [data/Maps/Portals.csv](../../data/Maps/Portals.csv)).
- `ActionType::ChangeMap` validates the player is actually near a portal
  (anti portal-hack, kick-back otherwise) and teleports across maps
  ([msg_action.rs:287](../../server/game/src/packets/msg_action.rs#L287),
  [world/portal.rs](../../server/game/src/world/portal.rs)).

## Weather

- Per-map weather state with kinds (rain, snow, etc.) broadcast via
  `MsgWeather` ([msg_weather.rs](../../server/game/src/packets/msg_weather.rs));
  changeable at runtime with the `$weather` GM command.

## Gaps

- Map flags (PK-free zones, etc.) are sent but not enforced anywhere.
- No dynamic/instanced maps, no map-based events.
- Region broadcast is the only chat scope (see [chat.md](../features/chat.md)).
