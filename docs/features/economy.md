# Economy (Shops, Warehouse, Booths, Banking) — ❌ Missing

## Current state

| Feature | What exists | Where |
|---|---|---|
| NPC shops | `NpcKind::ShopKeeper` modeled; `Buy`/`Sell` item actions enumerated but unhandled | [entities/npc.rs](../../server/game/src/entities/npc.rs), [msg_item.rs](../../server/game/src/packets/msg_item.rs) |
| Warehouse | Storage NPCs answer with an `OpenDialog` action and nothing else | [msg_npc.rs:71](../../server/game/src/packets/msg_npc.rs#L71) |
| Booths | `LeaveBooth` clears/reloads the screen; `CreateBooth`/`SuspendBooth`/`ResumeBooth` and the `BoothQuery/Add/Del/Buy/AddCPs` item actions are unhandled; `NpcKind::Booth`/`BoothFlag` modeled | [msg_action.rs:182](../../server/game/src/packets/msg_action.rs#L182) |
| Banking | `QueryMoneySaved` / `SaveMoney` / `DrawMoney` item actions enumerated, unhandled; no `money_saved` column on characters | [msg_item.rs](../../server/game/src/packets/msg_item.rs), [1_characters.sql](../../migrations/1_characters.sql) |
| Money on hand | Silver and CPs persist on the character and display | [character.rs:76](../../server/game/src/entities/character.rs#L76) |

## What's needed

Everything here is downstream of the item system
([items-and-inventory.md](../features/items-and-inventory.md)):

1. **Shop data** — `shop.dat`-equivalent (NPC id → item list) ingested into a
   table; then `Buy`/`Sell` with silver/CP transactions.
2. **Warehouse** — per-character, per-city storage table; dialog/window flow
   through the storage NPC.
3. **Banking** — `money_saved` column + the three bank item-actions.
4. **Booths** — booth session state in the market map, item listing, and
   purchase flow between two online players.
