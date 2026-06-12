# Current State

> Analysis date: 2026-06-11. The README's own banner says it best:
> "⚠ Still Under Construction ⚠". The login pipeline and world foundation are
> solid; gameplay systems are largely not started.

## What works end-to-end today

A 5017 client can:

1. **Log in** — the auth server authenticates against the `accounts` table
   (bcrypt via `tq-db`), resolves the realm, and hands off to the game server
   with a login token (`MsgAccount` → `MsgConnectEx`/`MsgTransfer`). The auth
   packet handlers run sandboxed in wasmtime.
2. **Create a character** — `MsgRegister` validates body type and base class
   (Trojan/Warrior/Archer/Taoist), rolls random avatar/hair, computes starting
   stats, and persists to SQLite
   ([msg_register.rs](../../server/game/src/packets/msg_register.rs)).
3. **Enter the world** — maps load from DB + DMap floor files, the client gets
   map info, weather, and ARGB color; spawn packets are exchanged with nearby
   players via the screen system.
4. **Move around** — walking/running ([msg_walk.rs](../../server/game/src/packets/msg_walk.rs)),
   jumping with anti-cheat validation (distance, elevation, position echo)
   ([msg_action.rs](../../server/game/src/packets/msg_action.rs#L191)), facing
   changes, and **portals** that teleport across maps
   ([msg_action.rs:287](../../server/game/src/packets/msg_action.rs#L287)).
5. **Chat** — messages broadcast to the local map region; `$`-prefixed GM
   commands (`dc`, `tele`, `which --map`, `jump-back`, `weather`) parsed with
   `argh` ([commands.rs](../../server/game/src/systems/commands.rs)).
6. **See NPCs** — NPCs load from DB per map and spawn into screens; clicking
   one shows a placeholder dialog
   ([msg_npc.rs](../../server/game/src/packets/msg_npc.rs)).
7. **Persist** — character location/stats are saved back to SQLite
   ([character.rs:190](../../server/game/src/entities/character.rs#L190)).

## Quality of the foundation

The infrastructure layer is the strongest part of the repo:

- **Networking**: actor-based connection handling (`tq-network`), TQ packet
  framing (`tq-codec`), and the full cipher suite for 5017 (`tq-crypto`).
- **Wire format**: a real serde data format for TQ packets (`tq-serde`) with
  fixed-length strings and password types — packet structs are plain
  `#[derive(Serialize, Deserialize, PacketID)]`.
- **World**: region-partitioned maps with weak-ref entity tracking, weather,
  portals, and DMap-derived tile access/elevation data
  ([map.rs](../../server/game/src/world/map.rs),
  [floor.rs](../../server/game/src/systems/floor.rs)).
- **Screen system**: surroundings loading, observer exchange, movement
  fan-out ([screen.rs](../../server/game/src/systems/screen.rs)).
- **WASM experiment**: the auth server already runs its two packets as
  isolated WASM modules with a typed host ABI — an unusual and interesting
  architecture for hot-reloadable/sandboxed packet logic. The recent `wasmify`
  commit (#86) suggests the game server is intended to follow.

## Where it stops

Anything past "walk around and look at things" is unimplemented: no items or
inventory, no combat, no monsters, no skills/magic, no trade, no teams,
no guilds, no shops/banking. Several `MsgAction` subtypes reply with an echo
plus an in-client "Missing Action Type" notice, and unknown `MsgItem` actions
do the same. The full inventory of gaps is in
[Missing & Roadmap](../analysis/missing-and-roadmap.md), and per-feature
status lives in the [Feature Matrix](../features/README.md).

## Known TODOs in code

- Chat is region-only; whisper/world/etc. unimplemented
  ([msg_talk.rs:144](../../server/game/src/packets/msg_talk.rs#L144))
- Kill mode is acknowledged but not stored or enforced
  ([msg_action.rs:324](../../server/game/src/packets/msg_action.rs#L324))
- `SendItems`/`SendAssociates`/`SendProficiencies`/`SendSpells`/`ConfirmGuild`
  are echo-only stubs ([msg_action.rs:354](../../server/game/src/packets/msg_action.rs#L354))
- `$tele --all` (teleport all) not implemented
  ([commands.rs:34](../../server/game/src/systems/commands.rs#L34))
- Portal-hack and NPC-interaction-hack detection log but don't punish
  ([msg_action.rs:295](../../server/game/src/packets/msg_action.rs#L295),
  [msg_npc.rs:66](../../server/game/src/packets/msg_npc.rs#L66))
- `send_spawn` panics (`todo!`) for non-character entities
  ([entities/mod.rs:65](../../server/game/src/entities/mod.rs#L65))
