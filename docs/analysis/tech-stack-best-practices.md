# Tech Stack Best Practices — Targeting 5,000 Concurrent Players

> Analysis date: 2026-06-11. Scope: persistence (PostgreSQL vs SQLite), caching
> (Redis or not), runtime/networking, and the operational practices needed for
> a server that 5,000 players can play on *continuously*. Grounded in the
> current codebase (tokio + sqlx/SQLite + actor-based `tq-network`).

## TL;DR recommendations

| Concern | Recommendation | Why |
|---|---|---|
| Database | **PostgreSQL** for production; keep SQLite for dev/tests | SQLite is single-writer; 5k CCU produces steady concurrent writes |
| Redis | **No — skip it** (for now) | Single authoritative game process: RAM *is* the cache. Redis adds a hop and a consistency problem for zero gain at this scale |
| Persistence pattern | **Write-behind with dirty flags** + transactional write-through for item/trade ops | DB is a persistence medium, not the source of truth |
| Runtime | **Keep tokio multi-thread** + actor-per-connection | 5k connections is well within tokio's comfort zone |
| Topology | **One game-server process per realm, one Postgres** | Don't build distributed-systems machinery you don't need |
| Availability | Graceful shutdown w/ full flush, systemd/container auto-restart, Postgres streaming replica + PITR backups | "Continuously available" is an ops problem more than a code problem |

## 1. Scale reality check — 5,000 CCU is modest

Before picking infrastructure, size the actual load. A Conquer-style 2D MMO at
5,000 CCU on one realm:

- **Network**: ~5,000 TCP connections, each sending a handful of small packets
  per second (movement, chat, actions). Tokio handles hundreds of thousands of
  connections; 5k is trivial. The real cost is **broadcast fan-out** (one move
  → N observers), which the region-partitioned screen system
  ([screen.rs](../../server/game/src/systems/screen.rs)) already bounds.
- **Database**: with write-behind saves every 60s per online character, that is
  ~83 row writes/second plus login/logout bursts. Postgres on a small instance
  does thousands of TPS without tuning.
- **Memory**: even a generous 100 KB of state per character is 500 MB. World +
  DMap floor data dominates, and it's shared, not per-player.
- **Hardware**: one box with 8–16 cores and 32 GB RAM has large headroom.
  The original 2003-era servers ran comparable populations on far less.

**Conclusion: this is a single-process, single-database problem.** Every
recommendation below follows from that. The classic failure mode at this scale
is over-architecting (microservices, message queues, Redis-as-state-store),
not under-architecting. See [ithare.com's classical MMO deployment
architecture](http://ithare.com/chapter-via-server-side-mmo-architecture-naive-and-classical-deployment-architectures/2/)
— the "classical" single-world-server design is exactly what coemu already is.

## 2. Database: yes, move to PostgreSQL

### Why SQLite stops being enough

SQLite is excellent for what it currently does here (dev loop, tests, seeded
game data), but it is architecturally single-writer:

- Even in WAL mode, **only one write transaction can run at a time**; under
  concurrent write load every other writer queues behind a file lock
  (`SQLITE_BUSY` retries). 5k players generate a continuous trickle of writes —
  character saves, account logins, (future) item/trade/guild mutations.
- Postgres uses MVCC: readers never block writers, writers don't block each
  other on different rows. This is precisely the workload shape of a game
  server in steady state.
- Postgres brings operational pieces "continuous availability" needs and
  SQLite cannot offer: **streaming replication, point-in-time recovery, online
  backups** while the server is running.

### Why Postgres over MySQL/MariaDB or NoSQL

- First-class sqlx support (already a workspace dependency), strong types
  (arrays, JSONB for flexible item attributes if needed), transactional DDL
  for migrations, the best tooling ecosystem (pgBackRest, pg_stat_statements).
- NoSQL buys nothing here: the schema is relational (accounts → characters →
  items), volumes are tiny by database standards, and **item/trade integrity
  wants ACID transactions** (the canonical MMO dupe-bug source is exactly
  non-transactional item movement).

### Concrete migration path

The good news: the codebase is already 90% portable because sqlx is used with
plain SQL everywhere. The work is mechanical:

1. **`tq-db` hard-codes `sqlx::SqlitePool` in every signature**
   ([character.rs](../../crates/db/src/character.rs),
   [account.rs](../../crates/db/src/account.rs), etc.). Either switch the type
   alias to `PgPool` behind the existing `sqlx` feature flag, or make the pool
   type a feature-selected alias (`type DbPool = sqlx::PgPool` /
   `sqlx::SqlitePool`) so dev/tests can stay on SQLite.
2. **Workspace features**: swap `sqlite` → `postgres` in the sqlx feature
   lists ([Cargo.toml](../../Cargo.toml), [server/game/Cargo.toml](../../server/game/Cargo.toml),
   [crates/db/Cargo.toml](../../crates/db/Cargo.toml)). Keep
   `runtime-tokio-rustls`. While touching this, **upgrade sqlx 0.7 → 0.8**
   (0.7 has an unpatched RUSTSEC advisory; 0.8 is the maintained line).
3. **Migrations** ([migrations/](../../migrations/)) are mostly portable SQL;
   audit for SQLite-isms (`AUTOINCREMENT` → `GENERATED ALWAYS AS IDENTITY`,
   type affinities → real types, `INTEGER` booleans → `BOOLEAN`).
4. **Pool sizing**: small. A single game process needs roughly
   `max_connections = cores × 2–4` (10–20). Do **not** size the pool to player
   count — 5,000 players share a handful of connections because queries are
   short. No PgBouncer needed: PgBouncer solves many-clients-few-connections,
   and there are only two clients (auth + game).
5. **Run Postgres on the same machine or same LAN** as the game server.
   Sub-millisecond round trips keep the async save paths cheap.

### Two-tier data split worth formalizing

- **Static game data** (maps, portals, NPCs, item prototypes, mob spawns —
  the `generated_*` migrations): load once at boot into immutable in-memory
  structures (`Arc`/`arc-swap`, as today). This data could even *stay* in
  SQLite or flat files permanently — it's read-only deployment data.
  Postgres is only truly required for **player data**.
- **Player data** (accounts, characters, items, guilds): Postgres, owned by
  the persistence layer below.

## 3. Redis: no — and here's the decision rule

Redis earns its place when **multiple processes need shared fast state**. A
single authoritative game-server process already holds the entire world in
RAM; putting a cache *next to* the source of truth is strictly worse:

- Every Redis read is a network round trip to data the process already has
  in a local `Arc` (~100 ns vs ~0.5 ms — three orders of magnitude).
- It creates a second copy of state that can drift (cache invalidation bugs)
  and a second service that can fail — directly hurting the "continuously
  available" goal.
- The standard web-app pattern "Postgres + Redis cache" exists to protect a
  DB from **read** traffic from many stateless app servers. A game server has
  near-zero steady-state DB reads: a character is read once at login and then
  lives in memory. There is nothing to cache.

This matches MMO-architecture practice: keep the source of truth in the
simulation process, treat the DB as a persistence medium
([PRDeving on MMO dataflows](https://prdeving.wordpress.com/2023/09/29/mmo-architecture-source-of-truth-dataflows-i-o-bottlenecks-and-how-to-solve-them/)).
Redis-as-world-state guides target *horizontally scaled, stateless* game
nodes — a different architecture with much worse latency characteristics,
needed only far beyond 5k CCU per shard.

**When Redis (or similar) becomes justified later:**

| Trigger | What Redis would do |
|---|---|
| Multiple realms needing shared presence (cross-realm chat, friends online) | Pub/sub + shared session registry |
| Auth and game servers on different machines at large scale | Login-token handoff store (today `MsgTransfer` tokens can live in Postgres or in-process just fine) |
| Cross-shard leaderboards | Sorted sets |
| A web account/ranking site hammering the DB | Classic read cache |

If an *in-process* cache is ever needed (e.g. name→id lookups), use
[`moka`](https://crates.io/crates/moka) (async LRU/TTL cache crate) — no new
infrastructure.

## 4. Persistence pattern: write-behind + transactional islands

This is the most important best practice for the 5k-CCU target, and it's a
code pattern rather than an infrastructure choice:

1. **In-memory state is authoritative** while a character is online (already
   true). The DB write on save ([character.rs:190](../../server/game/src/entities/character.rs#L190))
   is a checkpoint, not a transaction the gameplay waits on.
2. **Dirty-flag + periodic flush**: mark characters dirty on mutation; a
   background tokio task sweeps every 30–60 s and batches UPDATEs for dirty
   characters (chunked, e.g. 100 per transaction). Never write per-action or
   per-tick — per-action writes are the classic cost/latency/lock-contention
   trap ([techtidesolutions on persistence](https://techtidesolutions.com/blog/game-server-architecture-basics/)).
3. **Flush on logout and on graceful shutdown** — these two paths must be
   reliable; the periodic sweep just bounds data loss for crashes (≤ one
   sweep interval).
4. **Write-through transactions for value transfers**: when items, trade,
   mail, market, and guild banks arrive (per the
   [roadmap](missing-and-roadmap.md)), those mutations must be synchronous
   ACID transactions (`BEGIN … COMMIT`) — *not* write-behind. Item dupes come
   from crashing between two halves of a non-atomic transfer. This is the
   single strongest argument for Postgres over SQLite: these transactional
   writes will contend with the background sweep, which SQLite serializes
   and Postgres does not.
5. **Saves must not block the game loop**: spawn flush work onto separate
   tasks; the actor model already isolates per-connection processing.

## 5. Runtime & networking: the current stack is right

- **tokio (multi-thread) + actor-per-connection + `tq-codec` framing**: keep
  it. This is the mainstream Rust pattern for exactly this problem. No need
  for io_uring, thread-per-core frameworks (glommio/monoio), or a bespoke
  runtime at 5k connections.
- `parking_lot`, `arc-swap`, weak-ref entity tracking: all appropriate;
  short non-async critical sections with `parking_lot` are best practice.
- **Add backpressure**: bound per-connection outbound queues and disconnect
  (or drop low-priority packets for) clients that can't keep up, so one slow
  consumer can't balloon memory during broadcast storms.
- **Add admission control**: per-IP connection caps, login rate limiting
  ([`governor`](https://crates.io/crates/governor)), idle timeouts, and a
  hard CCU cap with a "server full" response — required for the availability
  goal under load spikes or floods.
- **bcrypt on the runtime**: password hashing is intentionally ~100 ms of CPU;
  run it via `tokio::task::spawn_blocking` in the auth path so login bursts
  don't stall the reactor (worth verifying in `tq-db`'s `Account::auth`).
- The **wasmtime packet sandboxing** is a genuine availability asset: a
  panicking handler can be contained, and hot-swapping packet-logic modules
  enables logic fixes without rebooting the world. Lean into it for the
  game server (the `wasmify` direction) — restarting a world server is the
  single biggest availability event an MMO has.

## 6. Continuous availability: the ops checklist

"5,000 players who can play continuously" is mostly operations:

- **Graceful shutdown** (`tokio::signal`, already a dependency): stop
  accepting connections → notify clients → flush all dirty state → exit.
  Pair with systemd `Restart=always` (or container restart policy) so crashes
  self-heal in seconds.
- **Postgres durability**: nightly base backups + WAL archiving
  (**pgBackRest**) for point-in-time recovery — the answer to "rollback after
  a dupe exploit" — plus one **streaming replica** for hardware failure.
  That is the entire HA story needed at this scale; skip Patroni/multi-master.
- **Observability**:
  - Keep `tracing`; add the [`metrics`](https://crates.io/crates/metrics)
    facade + `metrics-exporter-prometheus` and dashboard the four numbers
    that matter: CCU, tick/handler latency, dirty-queue depth & flush
    duration, DB pool wait time.
  - The `console-subscriber` feature (tokio-console) is already wired in for
    debugging task stalls — good.
  - A trivial TCP/HTTP health endpoint for the supervisor.
- **Deploys**: rolling restarts are impossible with one stateful world
  process, so (a) keep restarts rare and fast (flush + reboot in seconds at
  off-peak), and (b) use WASM hot-reload for logic-only changes. Realm
  selection via the `realms` table already gives a path to multiple worlds
  if one community outgrows a shard.
- **Anti-cheat is an availability feature**: the existing detections that
  "log but don't punish" ([msg_action.rs:295](../../server/game/src/packets/msg_action.rs#L295))
  should eventually disconnect — speed/portal hacks consume disproportionate
  broadcast and validation resources.

## 7. Recommended stack summary

| Layer | Choice | Status |
|---|---|---|
| Language/runtime | Rust + tokio multi-thread | ✅ in place |
| Networking | actor-per-connection, `tq-codec`/`tq-crypto` | ✅ in place |
| Database (players) | **PostgreSQL 16/17** via sqlx **0.8**, rustls | 🔁 migrate from SQLite |
| Database (dev/test) | SQLite behind a feature flag | ✅ keep |
| Static game data | Loaded to RAM at boot (DB or flat files) | ✅ in place |
| Cache | None (process RAM); `moka` if a need appears | ➕ nothing to add |
| Redis | Not used; revisit at multi-realm shared state | ❌ skip |
| Persistence pattern | Write-behind sweeps + ACID write-through for items/trades | ➕ to build |
| Rate limiting | `governor` at login/accept | ➕ to build |
| Metrics | `metrics` + Prometheus exporter + Grafana | ➕ to build |
| Backups/HA | pgBackRest (PITR) + 1 streaming replica | ➕ ops |
| Hot logic updates | wasmtime module reload | 🔁 in progress (`wasmify`) |

### Suggested order of adoption

1. sqlx 0.7 → 0.8 upgrade and make `tq-db` pool type pluggable.
2. Postgres migration (features, migrations audit, pool config) — *before*
   items/inventory land, since those want Postgres transactions from day one.
3. Dirty-flag write-behind sweep + logout/shutdown flush.
4. Backpressure + login rate limiting + CCU cap.
5. Metrics + dashboards; load-test with `tools/benchbot` toward 5k synthetic
   clients.
6. Ops: systemd unit, pgBackRest, replica.

## Sources

- [PostgreSQL vs SQLite — concurrency deep dive (dev.to)](https://dev.to/lovestaco/postgresql-vs-sqlite-dive-into-two-very-different-databases-5a90)
- [SQLite vs PostgreSQL benchmark analysis (tableone.dev)](https://tableone.dev/blog/sqlite-vs-postgresql-performance)
- [SQLite forum: SQLite for web-game data with many users](https://sqlite.org/forum/forumpost/c6fc2a1cc3?t=h&unf=&hist=)
- [IT Hare: Server-side MMO architecture — classical deployment](http://ithare.com/chapter-via-server-side-mmo-architecture-naive-and-classical-deployment-architectures/2/)
- [PRDeving: MMO architecture — source of truth, dataflows, I/O bottlenecks](https://prdeving.wordpress.com/2023/09/29/mmo-architecture-source-of-truth-dataflows-io-bottlenecks-and-how-to-solve-them/)
- [Tech Tide: Game server architecture basics (persistence batching)](https://techtidesolutions.com/blog/game-server-architecture-basics/)
- [GameDev.net: Eventual consistency in an MMO-ish game database](https://gamedev.net/forums/topic/690749-eventual-consistency-in-an-mmoish-game-database-structure/)
- [Redis docs: distributed caching (when shared state layers apply)](https://redis.io/glossary/distributed-caching/)
