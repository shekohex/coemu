# Combat & PK — ❌ Missing (kill modes 🟡 stubbed)

## Current state

There is **no combat at all**: no handler for `MsgInteract` (packet 1022),
which carries physical attacks, magic casts, archery, and court/marry actions
in 5017. No damage formulas, no attack speed validation, no aggro.

The only combat-adjacent code is kill modes:
`ActionType::SetKillMode` ([msg_action.rs:322](../../server/game/src/packets/msg_action.rs#L322))
accepts Free / Safe / Team / Arrestment and sends the right notice, but the
mode is neither stored on the character nor enforced (`TODO` in code).

Related unhandled actions: `Ghost` (137), `XpClear` (93), `Reborn` (94),
`QueryEnemyInfo` (123).

## What's needed

- **`MsgInteract` handler** with attack-type dispatch (physical / magic /
  archer) and server-side range, cooldown, and line-of-sight checks.
- **Damage model** — 5017 formulas from str/agi/equipment/level differences,
  plus dodge/defense; depends on items
  ([items-and-inventory.md](../features/items-and-inventory.md)) and stats.
- **Death flow** — HP 0 → death animation, ghost state (`Ghost` action,
  `TalkChannel::Ghost` already exists), revive at spawn, XP loss, item/money
  drops on PK death.
- **PK system** — kill-mode enforcement, PK points (the `kill_points` column
  already exists in [1_characters.sql](../../migrations/1_characters.sql)),
  name-flashing (blue/red/black names), guard aggression, and jail (the
  portal-hack TODO at
  [msg_action.rs:295](../../server/game/src/packets/msg_action.rs#L295) also wants
  jail).
- **Map flag enforcement** — PK-disabled maps are sent to the client but not
  enforced server-side.
