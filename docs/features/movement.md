# Movement — ✅ Implemented

All ground movement for player characters works, with server-side validation.

## Walking / running

`MsgWalk` (1005, [msg_walk.rs](../../server/game/src/packets/msg_walk.rs)):
direction-indexed offsets, tile access check (`tile.access > TileType::Npc`),
elevation tracking, region re-homing, and fan-out to observers via the screen
system. Invalid steps trigger a kick-back to the last valid position.

## Jumping

`ActionType::Jump` ([msg_action.rs:191](../../server/game/src/packets/msg_action.rs#L191))
validates, in order:

1. The client-reported current position matches the server's.
2. The target is within screen distance (≤18 tiles, `tq_math::in_screen`).
3. The elevation delta is acceptable (`Map::sample_elevation`, >210 rejected).
4. The destination tile is walkable.

Failures result in a kick-back; success updates location/direction/elevation
and broadcasts the jump.

## Facing & teleport

- `ActionType::ChangeFacing` with position-echo validation
  ([msg_action.rs:247](../../server/game/src/packets/msg_action.rs#L247)).
- `Character::teleport` ([character.rs:147](../../server/game/src/entities/character.rs#L147))
  moves between maps: removes from old map/screen, loads the target map's
  floor, validates the destination tile, then sends teleport + weather + map
  info.
- `Character::kick_back` snaps the client back to the server-side position —
  the standard anti-cheat response.

## Gaps

- No movement speed / stamina checks (run vs walk timing isn't validated).
- Mounts don't exist in 5017; not applicable.
- Monster/NPC movement doesn't exist (no AI; see
  [monsters-and-ai.md](../features/monsters-and-ai.md)).
