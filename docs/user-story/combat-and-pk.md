# Epic: Combat & PK

> Source: [combat-and-pk.md](../features/combat-and-pk.md) (❌ missing, kill
> modes stubbed), [MsgInteract/MsgUserAttrib notes](../notes/co-5017-protocol-notes.md).
> Damage formulas and PK constants are wiki stubs — source from COPSv6/Soul
> and keep them in one tunable module.

## CBT-1: Physical attacks

**As a** player **I want** to melee-attack monsters and players **so that**
the core combat loop exists.

**Acceptance criteria:**
- `MsgInteract` (1022, 32-byte 5017 layout) handler dispatches ATTACK (2);
  server validates target exists in screen, is alive, is attackable (map PK
  flags, kill mode), and is in melee range; coordinates from the packet are
  echoed but **server state is authoritative**.
- Damage computed server-side from attacker str/equipment vs defender
  defense/dodge with the 5017 level-gap modifiers; result broadcast as the
  interaction response with damage in `Data`.
- Attack-speed (interval) enforcement server-side; out-of-range or too-fast
  attacks are dropped silently and counted toward anti-cheat metrics.
- HP changes push via `MsgUserAttrib` LIFE to the target and observers.

## CBT-2: Ranged (archery) attacks

**As a** player **I want** bow attacks **so that** archers are playable.

**Acceptance criteria:**
- SHOOT (25) validated for bow equipped, arrows in inventory (consumed per
  shot), range, and line of sight over DMap tiles; damage uses the archery
  formula.

## CBT-3: Death and revival

**As a** player **I want** death, ghost state, and revival **so that** combat
has stakes.

**Acceptance criteria:**
- HP 0 → death broadcast, ghost flag (USERSTATUS_GHOST 0x400 via
  `MsgUserAttrib` USERSTATUS), `TalkChannel::Ghost` chat works as ghost.
- `Ghost` (137) and `Reborn` (94) `MsgAction` types handled: revive at map
  revive point (or spot-revive items later); ghosts can't act on the living.
- PvE death below the 5017 rules costs XP; PvP death applies the
  red/black-name drop rules (CBT-5).

## CBT-4: Kill modes enforced

**As a** player **I want** Free/Safe/Team/Arrestment kill modes to actually
gate my attacks **so that** I don't strike unintended targets.

**Acceptance criteria:**
- `SetKillMode` ([msg_action.rs:322](../../server/game/src/packets/msg_action.rs#L322))
  stores the mode on the character (resolving the code TODO) and every attack
  path consults it: Safe blocks players/blue-named appropriately, Team
  exempts teammates (and later guildmates), Arrestment only flashing/red/black
  names.
- Mode persists for the session; default on login is Safe for low levels.

## CBT-5: PK points and name flags

**As a** player **I want** unlawful kills to flag killers red/black
**so that** open-world PK has consequences.

**Acceptance criteria:**
- Unlawful player kills add PK points (persisted in the existing
  `kill_points` column); points decay over online time.
- Thresholds drive name color: ≥30 red (USERSTATUS_RED_NAME), ≥100 black
  (BLACK_NAME); the victim's killer gets FLASHING_NAME for the guard-aggro
  window; all via `MsgUserAttrib` USERSTATUS + PK attribute updates.
- Red/black deaths drop equipment per 5017 rules (count scales with PK
  level); dying clears the appropriate points.
- Map PK flags (already sent to the client) are enforced server-side; safe
  zones block attacks outright.

## CBT-6: Guards and jail

**As a** player **I want** city guards to punish criminals **so that** towns
are safe.

**Acceptance criteria:**
- Guard NPCs aggro flashing/red/black-name players in range using the
  monster AI loop with guard stats.
- GM/portal-hack/jail flows: the existing detections that "log but don't
  punish" ([msg_action.rs:295](../../server/game/src/packets/msg_action.rs#L295),
  [msg_npc.rs:66](../../server/game/src/packets/msg_npc.rs#L66)) gain a jail
  teleport consequence.

## CBT-7: XP skill (battle power) bar

**As a** player **I want** the XP circle to fill during combat and enable
XP skills **so that** sustained fighting is rewarded.

**Acceptance criteria:**
- Combat actions build XP; full bar sets USERSTATUS_XPFULL; `XpClear` (93)
  handled; XP-skill activation (Cyclone/Superman statuses 0x800000/0x40000)
  applies timed effects and clears the bar.
