# Missing Features & Suggested Roadmap

> Analysis date: 2026-06-11. "Missing" means no handler, no DB schema, and no
> system code exists; "partial" items are listed at the end.

## Missing gameplay systems

### Items & inventory (foundational — blocks almost everything else)
- No `items` table in [migrations/](../../migrations/); characters only persist
  stats/location/money.
- `MsgItemInfo` exists as a server→client packet
  ([msg_item_info.rs](../../server/game/src/packets/msg_item_info.rs)) but nothing
  generates inventories; `ActionType::SendItems` is an echo stub.
- All `MsgItem` action subtypes except `Ping` are unhandled
  ([msg_item.rs](../../server/game/src/packets/msg_item.rs)): buy, sell, drop, use,
  equip/unequip, split/combine, repair, improve/uplevel, enchant, booth ops,
  bank (save/draw money), fireworks.
- No equipment model, no `MsgItemInfoEx`-style equipment broadcast, no item
  drops (a `FloorItem` entity skeleton exists at
  [floor_item.rs](../../server/game/src/entities/floor_item.rs)).

### Combat
- No `MsgInteract` (packet 1022) handler at all — no physical attack, no magic
  attack, no archery, no PvP, no damage calculation.
- Kill mode is echoed but never stored/enforced
  ([msg_action.rs:322](../../server/game/src/packets/msg_action.rs#L322)); no PK
  points, no jail, no death/ghost/revive flow (`Ghost`/`XpClear` actions
  unhandled).

### Monsters & AI
- No monster/mob tables, no spawn generators, no AI loop, no respawn timers.
  Only stationary NPCs exist ([crates/db/src/npc.rs](../../crates/db/src/npc.rs)).
- `GameEntity::send_spawn` is `todo!()` for non-characters
  ([entities/mod.rs:65](../../server/game/src/entities/mod.rs#L65)) — spawning a
  monster would currently panic.

### Skills, magic & proficiencies
- No `MsgMagicInfo`, `MsgWeaponSkill`, or `MsgMagicEffect` packets; no skill
  tables; `SendProficiencies`/`SendSpells` are stubs.

### Progression
- No XP gain, level-up handling (`ActionType::LevelUp` unhandled), attribute
  point allocation (`MsgUserAttrib` missing), reborn/rebirth logic, or stat
  recalculation on level/class change.

### NPC dialogs, quests & shops
- NPC interaction sends a hard-coded demo dialog
  ([msg_npc.rs:84](../../server/game/src/packets/msg_npc.rs#L84)); `MsgTaskDialog`
  replies are not processed into any quest/task state machine.
- No shop inventories (`data/` has no itemtype/shop data), no vendor buy/sell,
  no warehouse/storage backend (storage NPCs just open an empty dialog).

### Social systems
- **Friends**: `SendAssociates` stub; no `MsgFriend` packet or table.
- **Team/party**: `MsgTeam`/`MsgTeamMember` missing; `QueryTeamMember` unhandled.
- **Guilds (syndicates)**: `ConfirmGuild` stub; no `MsgSyndicate`,
  `MsgSyndicateAttributeInfo`, or guild tables.
- **Trade**: no `MsgTrade` packet; player-to-player trading absent.
- **Whisper/world chat**: chat only broadcasts to the sender's map region
  ([msg_talk.rs:143](../../server/game/src/packets/msg_talk.rs#L143)).
- **Booths/market**: create/suspend/resume booth actions unhandled (only
  `LeaveBooth` clears the screen).

### Misc gameplay
- Mining (`ActionType::Mine`), transformation (`AbortTransform`), marriage
  (spouse channel exists in chat enum only), arena/tournaments, lottery/dice
  NPCs, i18n of system messages.

## Missing infrastructure

- **Game-server WASM migration**: auth packets run in wasmtime, but all 16
  game packets are native; the `wasmify` direction implies these should move
  to [packets/](../../packets/) crates eventually.
- **Test coverage**: only the auth runtime has meaningful integration tests;
  game packet handlers, screen system, and floor math are mostly untested.
- **Ops**: no Dockerfile/compose, no graceful-shutdown persistence sweep
  documented, no admin tooling beyond GM chat commands, no rate limiting /
  flood protection on packets.
- **Data pipeline**: item types, monster spawns, shop data, and level-up stat
  tables from the 5017 client are not ingested (only maps, portals, NPCs).

## Partial features (started, incomplete)

| Feature | Gap | Where |
|---|---|---|
| Chat | Region-only broadcast; channels ignored | [msg_talk.rs:144](../../server/game/src/packets/msg_talk.rs#L144) |
| GM commands | `tele --all` TODO; tiny command set | [commands.rs:34](../../server/game/src/systems/commands.rs#L34) |
| Kill mode | Not persisted/enforced | [msg_action.rs:324](../../server/game/src/packets/msg_action.rs#L324) |
| NPC interaction | Demo dialog only; storage NPC no-op | [msg_npc.rs](../../server/game/src/packets/msg_npc.rs) |
| Anti-cheat | Detects portal/NPC hacks, no enforcement | [msg_action.rs:295](../../server/game/src/packets/msg_action.rs#L295) |
| Floor items | Entity exists, never spawned/picked up | [floor_item.rs](../../server/game/src/entities/floor_item.rs) |
| Ping | Hacky timestamp offset, flagged in comments | [msg_item.rs:72](../../server/game/src/packets/msg_item.rs#L72) |

## Suggested implementation order

1. **Items**: `items` migration + `tq-db` model → inventory load on login
   (`SendItems` → `MsgItemInfo`) → equip/unequip → drop/pickup (floor items).
   Requires ingesting the client's itemtype data.
2. **Monsters**: spawn tables + generator loop → fix `send_spawn` for
   non-characters → basic wander AI.
3. **Combat**: `MsgInteract` physical attack → damage/death/XP → level-up and
   attribute points (`MsgUserAttrib`).
4. **NPC dialog engine**: task/dialog state machine → shops & warehouse on top
   of the item system.
5. **Social**: team → trade → friends → guild (each is mostly packet work once
   items/combat exist).
6. In parallel: port game packets to the WASM runtime if that remains the
   architectural goal, and grow integration tests alongside each system.
