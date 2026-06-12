# CoEmu Documentation

CoEmu is a Conquer Online 5017 server emulator written in Rust. It consists of
an **Auth (account) server** and a **Game server**, backed by SQLite, built as a
Cargo workspace.

## Contents

- [analysis/](analysis/) — State of the repository
  - [Repository Overview](analysis/repository-overview.md) — workspace layout, crates, and architecture
  - [Current State](analysis/current-state.md) — what is implemented and working today
  - [Missing & Roadmap](analysis/missing-and-roadmap.md) — gaps and suggested next steps
  - [Tech Stack Best Practices](analysis/tech-stack-best-practices.md) — DB/cache/runtime recommendations for 5k CCU
- [features/](features/) — Feature catalog with implementation status
  - [Feature Matrix](features/README.md) — every feature, at a glance
- [notes/](notes/) — External research notes
  - [Spirited's CO Documentation (5017)](notes/spirited-conquer-docs-5017.md) — source map and assessment
  - [CO 5017 Protocol Notes](notes/co-5017-protocol-notes.md) — distilled packet/data reference
- [user-story/](user-story/) — User stories for the missing gameplay systems
  - [Story Index](user-story/README.md) — epics, dependencies, delivery order
- [plan/](plan/) — Delivery planning
  - [Implementation Plan](plan/implementation-plan.md) — phases, per-story agent pipeline, risks
- [reviews/](reviews/) — Per-story teaching code reviews (written by the reviewer agent)

## Quick Facts

| | |
|---|---|
| Target client | Conquer Online patch **5017** |
| Language / toolchain | Rust (nightly, see [rust-toolchain.toml](../rust-toolchain.toml)) |
| Database | SQLite via `sqlx` (migrations in [migrations/](../migrations/)) |
| Servers | [server/auth](../server/auth/) (port handoff/login), [server/game](../server/game/) (world) |
| Packet sandboxing | Auth packets run as **WASM modules** via `wasmtime` |
| License | GPL-3.0-only |

## How the pieces fit

```
Client (5017)
   │  TQ/RC5 ciphers (crates/crypto)
   ▼
Auth Server (server/auth) ──── wasmtime runs packets/connect + packets/account
   │  MsgTransfer (login token)
   ▼
Game Server (server/game) ──── native packet handlers (server/game/src/packets)
   │
   ▼
SQLite (crates/db + migrations) + data files (data/GameMaps, data/Maps, data/NPCs.csv)
```
