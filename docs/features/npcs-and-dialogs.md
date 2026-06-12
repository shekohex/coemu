# NPCs & Dialogs — 🟡 Partial

## What works

- NPCs are stored in SQLite (`npcs` table,
  [crates/db/src/npc.rs](../../crates/db/src/npc.rs); seeded from
  [data/NPCs.csv](../../data/NPCs.csv) via
  [11_generated_npcs.sql](../../migrations/11_generated_npcs.sql)) and loaded per
  map at startup.
- The full 5017 NPC taxonomy is modeled in
  [entities/npc.rs](../../server/game/src/entities/npc.rs): `NpcKind` (shopkeeper,
  task, storage, forge, booth, city gate, …), `NpcSort`, `NpcBase`.
- NPCs spawn into player screens via `MsgNpcInfo`
  ([msg_npc_info.rs](../../server/game/src/packets/msg_npc_info.rs)).
- Clicking an NPC (`MsgNpc`, 2031 —
  [msg_npc.rs](../../server/game/src/packets/msg_npc.rs)) validates screen range
  (anti-hack) and responds: storage NPCs get an `OpenDialog` action, others
  get a `MsgTaskDialog` built with a fluent builder
  ([msg_task_dialog.rs](../../server/game/src/packets/msg_task_dialog.rs)) —
  text, input fields, options, avatar.

## What's missing

- **The dialog is a hard-coded demo** ("Hello, My name is …"). There is no
  task/quest engine mapping NPC IDs to dialog trees, and client replies to
  `MsgTaskDialog` are not processed into any state machine.
- **No NPC behaviors**: shopkeepers have no shop inventory, storage NPCs open
  nothing (no warehouse backend), forges/composers do nothing.
- **No scripting** — given the auth server's wasmtime architecture, WASM-based
  NPC scripts would be a natural fit, but nothing exists yet.
- Screen-range cheaters are detected but not punished
  ([msg_npc.rs:66](../../server/game/src/packets/msg_npc.rs#L66)).
