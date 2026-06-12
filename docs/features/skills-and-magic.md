# Skills, Magic & Proficiencies — ❌ Missing

## Current state

Nothing is implemented. The login sequence acknowledges the client's requests
but sends no data — all three are echo stubs in
[msg_action.rs](../../server/game/src/packets/msg_action.rs#L365):

- `ActionType::SendProficiencies` (77) — weapon skills; needs `MsgWeaponSkill`
- `ActionType::SendSpells` (78) — magic list; needs `MsgMagicInfo`
- `DropMagic` (109) / `DropSkill` (110) are enumerated, unhandled

There are no skill/magic packets, no `magictype` data, no proficiency or
spell tables in [migrations/](../../migrations/), and no casting logic.

## What's needed

1. **Data** — ingest the 5017 `magictype.dat` (spell levels, MP cost, power,
   range, milestones) and weapon-skill XP curves.
2. **Schema** — `spells` and `weapon_skills` tables keyed by character.
3. **Packets** — `MsgMagicInfo`, `MsgWeaponSkill` (login bursts), and
   `MsgMagicEffect` (cast broadcast with target list).
4. **Mechanics** — casting via `MsgInteract` magic attack type: MP cost,
   range/AoE target resolution, damage/heal application, spell leveling;
   weapon proficiency XP on melee hits; class skill learning (XP skills,
   Taoist spell quests).

Depends on combat ([combat-and-pk.md](../features/combat-and-pk.md)) and
benefits from monsters existing first.
