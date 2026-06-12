# Repository Overview

> Analysis date: 2026-06-11, branch `master` (commit `a9fde82`).

CoEmu is a Cargo workspace ([Cargo.toml](../../Cargo.toml)) targeting the Conquer
Online 5017 client. The workspace is organized into five groups: shared
`crates/`, network `packets/`, `server/` binaries, proc-`macros/`, and CLI
`tools/`.

## Servers

### Auth server — [server/auth](../../server/auth/)

The account/login server. Its distinguishing design choice is that **packet
handlers are compiled to WebAssembly and executed in `wasmtime`**
([lib.rs](../../server/auth/src/lib.rs)):

- `Runtime` holds a wasmtime `Engine`, a `Linker<State>`, and the compiled
  `msg_connect` / `msg_account` WASM modules.
- Incoming packets are written into the module's linear memory and dispatched
  to an exported `process_packet` function.
- Host functions are exposed to the modules in
  [linker.rs](../../server/auth/src/linker.rs): logging, randomness, actor
  send/shutdown/set-id, DB account auth, realm lookup, and a server bus
  (`check` / `transfer`) used to hand the player off to the game server.

### Game server — [server/game](../../server/game/)

The world server. Packet handlers here are **native Rust** (not WASM yet),
registered in [packets/mod.rs](../../server/game/src/packets/mod.rs). Major modules:

| Module | Purpose |
|---|---|
| [state/](../../server/game/src/state/mod.rs) | Global `State`: SQLite pool, loaded maps, login/creation tokens, entity registry |
| [entities/](../../server/game/src/entities/mod.rs) | `Character`, `Npc`, `FloorItem`, shared `BaseEntity` |
| [world/](../../server/game/src/world/map.rs) | `Map` (regions, tiles, weather, portals), `Portal` |
| [systems/](../../server/game/src/systems/mod.rs) | `Screen` (visibility), `Floor` (tile/elevation data), GM `commands` |
| [packets/](../../server/game/src/packets/mod.rs) | 16 packet types handled (see [Feature Matrix](../features/README.md)) |

## Shared crates ([crates/](../../crates/))

| Crate | Role |
|---|---|
| `tq-network` | Actor model over TCP, packet handler/encode/decode traits |
| `tq-codec` | TQ packet framing codec |
| `tq-crypto` | Ciphers: TQ cipher, CQ cipher, RC5, NOP ([crates/crypto](../../crates/crypto/)) |
| `tq-serde` | Serde (de)serializer for the TQ binary wire format, fixed strings, password hashing types |
| `tq-db` | SQLite models: account, character, realm, map, portal, npc ([crates/db](../../crates/db/)) |
| `tq-server` | TCP server scaffolding |
| `tq-math` | Game math: screen distance, direction sectors, circles |
| `primitives` | `Location`, `Point`, `Size`, etc. |
| `tq-bindings` | Host/guest bindings for the WASM packet ABI ([crates/bindings](../../crates/bindings/)) |
| `tq-wasm-builder` | Builds packet crates to WASM at compile time (used by `build.rs` in packet crates) |
| `tracing-wasm` | Tracing subscriber bridging WASM guest logs to the host |

## Packet crates ([packets/](../../packets/))

These are the WASM-compiled packet handlers used by the auth server:

- `msg-connect` (1052), `msg-account` (1051) — have `build.rs` + WASM builds
- `msg-connect-ex` — server→client login responses / rejection codes
- `msg-transfer` (1086 area) — auth→game handoff token

## Macros ([macros/](../../macros/))

`derive-packetid`, `derive-packethandler`, `derive-packetprocessor` — derive
macros for the packet traits.

## Tools ([tools/](../../tools/))

- `benchbot` — load-testing bot client (excluded from default workspace build)
- `gamemap-decoder` — decodes TQ `GameMap.dat`/DMap files
- `hash-pwd` — password hash utility
- `externref` — WASM externref post-processing

## Data & persistence

- [migrations/](../../migrations/) — 11 SQLite migrations: accounts, characters,
  realms, maps, portals, npcs + generated seed data. **Notably absent: items,
  monsters/mob spawns, skills/magic, guilds, friends** (see
  [Missing & Roadmap](../analysis/missing-and-roadmap.md)).
- [data/](../../data/) — `Maps/Maps.csv`, `Maps/Portals.csv`, `NPCs.csv`;
  `GameMaps/` holds client DMap files fetched via
  [scripts/fetch-test-fixtures.bash](../../scripts/fetch-test-fixtures.bash).

## Tooling & CI

- CI: [.github/workflows](../../.github/workflows/) — `ci.yml`, `lints.yml`, `audit.yml`
- Nix flake ([flake.nix](../../flake.nix)), `.env.example`, rustfmt config,
  cargo aliases (`cargo auth`, `cargo game` per [README.md](../../README.md))
- Tests: auth runtime has integration tests
  ([server/auth/src/lib.rs](../../server/auth/src/lib.rs#L134)); game server has
  [test_utils.rs](../../server/game/src/test_utils.rs) but thin coverage overall.
