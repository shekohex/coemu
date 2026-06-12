# Epic: Skills & Magic

> Source: [skills-and-magic.md](../features/skills-and-magic.md) (❌ missing),
> [MsgMagicInfo/MsgMagicEffect/MsgInteract notes](../notes/co-5017-protocol-notes.md).
> `magictype.dat` format is a wiki stub — spell data will come from a
> COPSv6/Soul-derived dataset.

## MAG-1: Spell catalogue and learned spells

**As a** player **I want** my learned spells listed when I log in
**so that** my skill bar works.

**Acceptance criteria:**
- `magictype` data (spell id, levels, MP cost, power, range, AoE shape,
  required level/profession, exp per level) seeded into a table + in-memory
  catalogue; `spells` table keyed by character (type, level, exp).
- `ActionType::SendSpells` (78, echo stub at
  [msg_action.rs](../../server/game/src/packets/msg_action.rs#L365)) sends one
  `MsgMagicInfo` (1103: exp, type, level) per learned spell.
- Learning sources work: levelling milestones and skill books (item use).

## MAG-2: Cast offensive magic

**As a** player **I want** to cast damage spells **so that** Taoists and
hybrid builds function.

**Acceptance criteria:**
- `MsgInteract` MAGIC_ATTACK (21) handler **decodes the 5017 obfuscation**
  (bit-rotate/XOR keyed on sender id, per the
  [protocol notes](../notes/co-5017-protocol-notes.md#msginteract-1022--combat--social-interactions-5017-layout))
  before validation.
- Server validates: spell learned, MP cost available (deducted via
  `MsgUserAttrib` MANA), target/cell in range, cooldown elapsed.
- Single-target and AoE resolution against screen entities; results broadcast
  as `MsgMagicEffect` (1105) with per-target `{role, damage}` entries.
- Magic damage uses the magic attack/defense formula; spell exp accrues per
  cast and levels the spell at catalogue milestones (`MsgMagicInfo` update).

## MAG-3: Support magic

**As a** player **I want** heals, buffs, and cures **so that** support play
works.

**Acceptance criteria:**
- Heal/buff spells target allies (self, team, anyone per spell rules); HP
  restore via `MsgUserAttrib`; timed statuses (shield, stigma, invisible,
  fly) set/clear the correct USERSTATUS bits with server-side expiry.
- Revive (Taoist) restores a ghost at the corpse per 5017 rules.

## MAG-4: Weapon proficiencies

**As a** player **I want** weapon skills that level with use **so that**
melee specialisation matters.

**Acceptance criteria:**
- `weapon_skills` table (character, type, level, exp);
  `ActionType::SendProficiencies` (77) sends `MsgWeaponSkill` (1025) per
  proficiency at login.
- Melee hits with a weapon type accrue proficiency exp
  (`weaponskilllevelexp` curve); level-ups push a `MsgWeaponSkill` update and
  feed the damage formula.

## MAG-5: Magic traps and ground effects

**As a** player **I want** trap-style spells to appear on the floor
**so that** area-denial spells render correctly.

**Acceptance criteria:**
- Trap casts create floor effects via `MsgMapItem` CAST_TRAP/SYNCHRO_TRAP/
  DROP_TRAP with trap ids from the magic-trap range (900,001–989,999);
  periodic damage applies to entities entering the cells.
