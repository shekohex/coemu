# Implementation Plan — User Stories → Working 5017 Server

> Created 2026-06-11. Operationalizes the [user stories](../user-story/README.md)
> using a three-agent delivery pipeline defined in
> [.claude/agents/](../../.claude/agents/). Companion docs:
> [tech-stack-best-practices.md](../analysis/tech-stack-best-practices.md)
> (infrastructure), [co-5017-protocol-notes.md](../notes/co-5017-protocol-notes.md)
> (protocol reference).

## The team

Three dedicated agents live in `.claude/agents/` and are invoked per story:

| Agent | Role | Invocation |
|---|---|---|
| [staff-engineer](../../.claude/agents/staff-engineer.md) | Senior staff Rust engineer — implements the story: code, migrations, unit tests; verifies build + tests before reporting | "use staff-engineer to implement ITM-1" |
| [qa-engineer](../../.claude/agents/qa-engineer.md) | Senior QA — independently verifies against acceptance criteria; writes adversarial/integration tests; per-criterion PASS/FAIL/UNTESTED verdict; reports defects, never fixes production code | "use qa-engineer to verify ITM-1" |
| [rust-mentor-reviewer](../../.claude/agents/rust-mentor-reviewer.md) | Reviewer + Rust mentor — reviews every change for correctness/security/idiom **and** writes a teaching review for a senior TypeScript engineer learning Rust; saves to `docs/reviews/` | "use rust-mentor-reviewer to review ITM-1" |

## Pipeline per story

Every story flows through the same five gates on its own branch
(`story/<id>-<slug>`):

```
1. IMPLEMENT   staff-engineer: code + unit tests, cargo check/clippy/test green
2. VERIFY      qa-engineer: per-criterion verdict; REJECT → back to gate 1
                with the defect list (loop until APPROVE)
3. REVIEW      rust-mentor-reviewer: REQUEST_CHANGES → back to gate 1;
                APPROVE → review saved to docs/reviews/<story>.md
4. HUMAN       you read the teaching review (this is the Rust course),
                spot-check in the 5017 client when client-visible
5. MERGE       update the story file (criteria → done) and the
                feature matrix status; merge to master
```

Rules of engagement:

- **One story in the pipeline at a time** per dependency chain; stories from
  independent epics (e.g. SOC-1 and ITM-2) may run in parallel on separate
  branches/worktrees.
- QA and the reviewer take the *story* as ground truth, not the
  implementation. Disagreements about a criterion's meaning bubble up to you
  rather than being silently resolved.
- A story is **done** only when: all criteria implemented or explicitly
  re-scoped in the story file, QA verdict APPROVE, review verdict APPROVE,
  docs updated.
- Client-visible stories get a manual smoke test against a real 5017 client
  before merge (login, perform the feature, relog, verify persistence).

## Phase 0 — Foundations (before the first gameplay story)

Small, unblocking work the pipeline itself depends on. Order matters:

| # | Work | Why first |
|---|---|---|
| 0.1 | **PRG-1: `MsgUserAttrib`** | Prerequisite for items, combat, economy — nothing can show a stat change without it. Smallest possible pipeline dry-run to shake out the three-agent loop |
| 0.2 | **sqlx 0.7 → 0.8 + pluggable pool type in `tq-db`** | Items (ITM-2) introduces the first multi-writer transactional tables; per the [tech-stack analysis](../analysis/tech-stack-best-practices.md), Postgres should land before the item economy, and the RUSTSEC fix shouldn't wait |
| 0.3 | **Server tick loop skeleton** | MON-3 (AI), CBT (attack intervals), MAG (status expiry), PRG-5 (mining) all need a tick; build it once, region-gated, with a tick-duration metric |
| 0.4 | **ID allocators** | Per-range entity ID allocation with recycling (monsters/traps) + non-predictable item IDs ([identifier notes](../notes/co-5017-protocol-notes.md#entity-identifier-ranges)) |
| 0.5 | **Write-behind save sweep** | Dirty-flag + periodic flush + logout/shutdown flush, so gameplay stories never add per-action DB writes |

## Phases 1–7 — Story schedule

Mirrors the dependency order in the [story index](../user-story/README.md):

| Phase | Stories | Outcome | Gate to next phase |
|---|---|---|---|
| 1 | ITM-1 → ITM-5 | Items exist: catalogue, persistence, login inventory, equip, consumables | Player can equip gear and drink a potion |
| 2 | SOC-1, SOC-2 (parallel with Phase 1 — no deps) | Whisper + friends; global online registry built | Two players can whisper cross-map |
| 3 | MON-1 → MON-3 | Monsters spawn and behave; `send_spawn` `todo!` resolved; tick loop proven | Maps populated, AI cost measured at load |
| 4 | CBT-1 → CBT-4, PRG-2, PRG-3, MON-4, MON-5, ITM-6 | The core loop: kill → loot → XP → level → allocate stats | A character can level by hunting |
| 5 | ECO-1 → ECO-3, ITM-7 | Shops, warehouse, banking, repair — silver sources meet sinks | Economy round-trips silver |
| 6 | MAG-1 → MAG-4 | Spells and proficiencies; magic-attack deobfuscation | All four base classes playable |
| 7 | ECO-4 → ECO-7, ITM-8, CBT-5 → CBT-7, MAG-5, PRG-4, PRG-5, SOC-3 → SOC-7 | Trade, booths, PK consequences, guilds, marriage, rebirth, mining | Feature matrix ✅ for 5017 core |

Re-planning is expected: after each phase, revisit the next phase's stories
against what was learned (especially Phase 4 — damage formulas come from
COPSv6/Soul references, and the first PvP tests will recalibrate them).

## Verification & tooling

- **Per story**: `cargo check/clippy/test --workspace` (staff-engineer),
  adversarial tests (qa-engineer), `cargo fmt` before merge.
- **Per phase**: extend `tools/benchbot` to script the new feature
  (login → equip → fight …) and run a load smoke test; watch the Phase 0.3
  tick metric and DB pool wait times against the 5k-CCU budget in the
  [tech-stack analysis](../analysis/tech-stack-best-practices.md).
- **Reviews as curriculum**: `docs/reviews/` accumulates one teaching review
  per story — successive reviews go deeper rather than repeating concepts.

## Risks & mitigations

| Risk | Mitigation |
|---|---|
| Wiki stubs: damage formulas, PK constants, magictype/monstertype data | Source from COPSv6/Soul-derived datasets before Phase 4/6; isolate all formulas in one tunable module so corrections don't ripple |
| 5017 client behavior differs from docs (several layouts marked ❓ unverified) | Manual client smoke test per client-visible story; packet-capture against the real client when a layout is suspect |
| `wasmify` direction (game-server packets → WASM) colliding with new handlers | Keep handlers thin (parse → call system code); systems stay native so a later WASM move only relocates the parse layer |
| Agent loop stalls (QA↔implement ping-pong) | Cap at 2 REJECT cycles, then escalate to you with both sides' positions |
| Economy dupes despite tests | ECO-7 audit logging lands *with* Phase 5, not after; PITR backups (tech-stack plan) are the last-resort rollback |
