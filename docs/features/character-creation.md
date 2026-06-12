# Character Creation & Persistence — ✅ Implemented

## Creation

Handled by `MsgRegister` (1001) in
[msg_register.rs](../../server/game/src/packets/msg_register.rs):

- Validates body type (male/female meshes) and base class
  (Trojan / Warrior / Archer / Taoist).
- Rolls a random avatar (gender-appropriate range) and hair style.
- Computes starting stats — Taoists get spirit 10 / strength 2, others
  strength 4; HP = 3·(str+agi+spi) + 24·vit, MP = 5·spi.
- New characters start in map **1010** (61,109) with 1000 silver.
- Rejects taken names (`MsgTalk::register_name_taken`) and requires a valid
  creation token issued during login.

## Persistence

- Schema: [migrations/1_characters.sql](../../migrations/1_characters.sql) — mesh,
  avatar, hair, silver, CPs, class/previous class, rebirths, level, XP,
  map/x/y, virtue, str/agi/vit/spi, attribute points, HP/MP, kill points.
- Model: [crates/db/src/character.rs](../../crates/db/src/character.rs).
- Runtime wrapper: [entities/character.rs](../../server/game/src/entities/character.rs)
  wraps the DB row with atomics and writes it back in `Character::save`
  ([character.rs:190](../../server/game/src/entities/character.rs#L190)).
- `MsgUserInfo` ([msg_user_info.rs](../../server/game/src/packets/msg_user_info.rs))
  sends the character sheet to the client on login; `MsgPlayer`
  ([msg_player.rs](../../server/game/src/packets/msg_player.rs)) spawns the
  character for other players.

## Gaps

- No inventory/equipment/skills are created or loaded (those systems don't
  exist yet).
- Character deletion (`ActionType::DelRole`) is not handled.
- Stat recalculation on level/class change is absent (no progression system).
