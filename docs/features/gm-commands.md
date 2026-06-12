# GM / In-Game Commands — 🟡 Partial

Chat messages prefixed with `$` are parsed as commands using `argh`
([systems/commands.rs](../../server/game/src/systems/commands.rs)). Bad input gets
the auto-generated usage text sent back through system chat.

## Available commands

| Command | Effect |
|---|---|
| `$dc` | Disconnect from the server |
| `$tele <map_id> <x> <y>` | Teleport to a map/location (`--all` is a TODO) |
| `$which --map` | Print current map name and ID |
| `$jump-back` | Snap back to the server-side position |
| `$weather <kind>` | Change the current map's weather |

## Gaps

- **No permission system** — every player can run every command; there is no
  GM flag on accounts or characters.
- `$tele --all` (bring all players) is unimplemented
  ([commands.rs:34](../../server/game/src/systems/commands.rs#L34)).
- The usual emulator admin set is absent: item/money spawning, level setting,
  kick/ban, broadcast, mob spawning, invisibility — most blocked on the
  corresponding gameplay systems not existing yet.
