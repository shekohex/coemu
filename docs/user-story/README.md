# User Stories

User stories for closing the gameplay gaps in coemu, derived from the
[feature matrix](../features/README.md) and the patch-5017 research in
[docs/notes/](../notes/spirited-conquer-docs-5017.md). Created 2026-06-11.

Each story follows the format: **As a** \<role\> **I want** \<capability\>
**so that** \<value\>, with acceptance criteria that reference the 5017
protocol details in [co-5017-protocol-notes.md](../notes/co-5017-protocol-notes.md).
Anti-cheat criteria flagged ⚠ come from explicit warnings on the
[Conquer Online Development Wiki](https://conquer-online.github.io/wiki/).

Roles used: **player**, **GM** (game master), **operator** (server admin).

## Epics

| Epic | Stories | Depends on |
|---|---|---|
| [Items & Inventory](items-and-inventory.md) | ITM-1 … ITM-8 | — (foundation) |
| [Economy & Trade](economy-and-trade.md) | ECO-1 … ECO-7 | Items |
| [Monsters & AI](monsters-and-ai.md) | MON-1 … MON-5 | Items (drops) |
| [Combat & PK](combat-and-pk.md) | CBT-1 … CBT-7 | Items, Monsters |
| [Skills & Magic](skills-and-magic.md) | MAG-1 … MAG-5 | Combat |
| [Progression](progression.md) | PRG-1 … PRG-5 | Combat/Monsters (XP) |
| [Social & Communication](social.md) | SOC-1 … SOC-7 | varies (friends/whisper have none) |

## Suggested delivery order

1. **ITM-1 … ITM-5** — item data, schema, inventory on login, equip, use.
   Nearly everything else is blocked on items.
2. **SOC-1, SOC-2** — whisper + friends: cheapest player-visible wins, no
   dependencies, exercises a global online-player registry needed later.
3. **MON-1 … MON-3** — monster data, spawning, basic AI (gives the world life
   and an XP source).
4. **CBT-1 … CBT-4** — physical combat, death, XP gain (with PRG-1/PRG-2 in
   the same arc).
5. **ECO-1 … ECO-3** — shops/warehouse/banking (silver sinks once drops exist).
6. **MAG-1 … MAG-3** — proficiencies and spells.
7. Remaining PK, guild, booth, marriage, rebirth stories.

## Conventions for acceptance criteria

- "Persisted" means surviving a server restart via the dirty-flag/write-behind
  rules in [tech-stack-best-practices.md](../analysis/tech-stack-best-practices.md);
  value transfers (trade, bank, booth, purchases) must be atomic transactions.
- All client-supplied values are revalidated server-side (range, ownership,
  proximity, funds) — the client is never trusted.
- New entity IDs respect the 5017 identifier ranges
  ([protocol notes](../notes/co-5017-protocol-notes.md#entity-identifier-ranges)).
