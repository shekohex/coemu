# Screen / Visibility System — ✅ Implemented

The "screen" is each player's view of nearby entities — who can see whom, and
who receives your movement packets. Implemented in
[systems/screen.rs](../../server/game/src/systems/screen.rs) (~450 lines).

## What it does

- **`load_surroundings`** — after login, teleport, or jump, scans the
  surrounding map regions and exchanges spawn packets (`MsgPlayer`) with every
  character in screen range
  (`Character::exchange_spawn_packets`,
  [character.rs:176](../../server/game/src/entities/character.rs#L176)).
- **`send_movement`** — fans a movement packet (walk/jump/facing) out to all
  observers, adding/removing entities from each other's screens as they cross
  the 18-tile screen boundary (`tq_math::in_screen`).
- **`send_message`** — broadcasts an arbitrary packet to current observers.
- **`remove_from_observers` / `clear`** — tears down visibility on teleport,
  disconnect, or booth exit.
- NPCs spawn into screens via `MsgNpcInfo`
  ([msg_npc_info.rs](../../server/game/src/packets/msg_npc_info.rs)).

The screen holds weak references to observed entities and is attached to the
actor's `Character` (`Character::set_screen`), avoiding reference cycles.

## Gaps

- Only `Character` entities exchange spawns; `GameEntity::send_spawn` is
  `todo!()` for monsters/floor items
  ([entities/mod.rs:65](../../server/game/src/entities/mod.rs#L65)).
- No spawn packet for floor items (`MsgMapItem` missing), so item drops can't
  be shown yet.
