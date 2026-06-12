# Feature Matrix

Status of every feature relevant to a Conquer Online 5017 emulator, as of
2026-06-11.

Legend: ✅ Implemented · 🟡 Partial · ❌ Missing

## Account & login

| Feature | Status | Details |
|---|---|---|
| Account authentication (bcrypt) | ✅ | [authentication.md](../features/authentication.md) |
| Realm lookup & auth→game handoff | ✅ | [authentication.md](../features/authentication.md) |
| 5017 ciphers (TQ/CQ/RC5) | ✅ | [authentication.md](../features/authentication.md) |
| WASM-sandboxed packet handlers (auth) | ✅ | [authentication.md](../features/authentication.md) |
| Character creation | ✅ | [character-creation.md](../features/character-creation.md) |
| Character persistence (save/load) | ✅ | [character-creation.md](../features/character-creation.md) |

## World

| Feature | Status | Details |
|---|---|---|
| Map loading (DB + DMap floor files) | ✅ | [world-and-maps.md](../features/world-and-maps.md) |
| Portals between maps | ✅ | [world-and-maps.md](../features/world-and-maps.md) |
| Map regions & broadcast | ✅ | [world-and-maps.md](../features/world-and-maps.md) |
| Weather | ✅ | [world-and-maps.md](../features/world-and-maps.md) |
| Walking / running | ✅ | [movement.md](../features/movement.md) |
| Jumping (with validation) | ✅ | [movement.md](../features/movement.md) |
| Teleport & kick-back | ✅ | [movement.md](../features/movement.md) |
| Screen / visibility system | ✅ | [screen-visibility.md](../features/screen-visibility.md) |
| Floor items (ground drops) | 🟡 | [items-and-inventory.md](../features/items-and-inventory.md) |

## Communication & admin

| Feature | Status | Details |
|---|---|---|
| Local (region) chat | 🟡 | [chat.md](../features/chat.md) |
| Whisper / world / channel routing | ❌ | [chat.md](../features/chat.md) |
| GM commands (`$dc`, `$tele`, …) | 🟡 | [gm-commands.md](../features/gm-commands.md) |
| Ping measurement | 🟡 | [chat.md](../features/chat.md) |

## NPCs & quests

| Feature | Status | Details |
|---|---|---|
| NPC loading & spawning | ✅ | [npcs-and-dialogs.md](../features/npcs-and-dialogs.md) |
| NPC dialogs (`MsgTaskDialog`) | 🟡 | [npcs-and-dialogs.md](../features/npcs-and-dialogs.md) |
| Quest / task engine | ❌ | [npcs-and-dialogs.md](../features/npcs-and-dialogs.md) |
| Shops & vendors | ❌ | [economy.md](../features/economy.md) |
| Warehouse / storage | ❌ | [economy.md](../features/economy.md) |

## Gameplay (mostly missing)

| Feature | Status | Details |
|---|---|---|
| Items & inventory | ❌ | [items-and-inventory.md](../features/items-and-inventory.md) |
| Equipment | ❌ | [items-and-inventory.md](../features/items-and-inventory.md) |
| Combat (melee/ranged/magic) | ❌ | [combat-and-pk.md](../features/combat-and-pk.md) |
| PK system & kill modes | 🟡 | [combat-and-pk.md](../features/combat-and-pk.md) |
| Death / ghost / revive | ❌ | [combat-and-pk.md](../features/combat-and-pk.md) |
| Monsters & AI | ❌ | [monsters-and-ai.md](../features/monsters-and-ai.md) |
| Skills, magic & proficiencies | ❌ | [skills-and-magic.md](../features/skills-and-magic.md) |
| XP / leveling / attribute points | ❌ | [progression.md](../features/progression.md) |
| Reborn / rebirth | ❌ | [progression.md](../features/progression.md) |
| Mining | ❌ | [progression.md](../features/progression.md) |

## Social & economy (missing)

| Feature | Status | Details |
|---|---|---|
| Player trade | ❌ | [social.md](../features/social.md) |
| Team / party | ❌ | [social.md](../features/social.md) |
| Friends / associates | ❌ | [social.md](../features/social.md) |
| Guilds (syndicates) | ❌ | [social.md](../features/social.md) |
| Marriage | ❌ | [social.md](../features/social.md) |
| Booths / player market | ❌ | [economy.md](../features/economy.md) |
| Banking (save/draw money) | ❌ | [economy.md](../features/economy.md) |
