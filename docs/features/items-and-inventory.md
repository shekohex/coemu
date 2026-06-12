# Items, Inventory & Equipment — ❌ Missing

This is the largest gap in the emulator and blocks most other gameplay
(combat, shops, trade, drops).

## What exists

- `MsgItemInfo` ([msg_item_info.rs](../../server/game/src/packets/msg_item_info.rs))
  — the server→client packet for describing an item — is defined but never
  populated from real data.
- `MsgItem` (1009, [msg_item.rs](../../server/game/src/packets/msg_item.rs))
  enumerates all 5017 item actions (buy, sell, drop, use, equip, unequip,
  split, combine, bank save/draw, repair, improve, uplevel, enchant, booth
  ops, fireworks) but **only `Ping` is handled** — everything else echoes and
  prints "Missing Item Action Type" in-client.
- A `FloorItem` entity skeleton exists
  ([entities/floor_item.rs](../../server/game/src/entities/floor_item.rs), 27
  lines) but nothing spawns, drops, or picks up items.
- `ActionType::SendItems` (inventory on login) is an echo stub
  ([msg_action.rs:354](../../server/game/src/packets/msg_action.rs#L354)).

## What's needed

1. **Item type data** — ingest the 5017 client's `itemtype.dat` (or a CSV
   equivalent) into `data/` + a migration, mirroring how maps/NPCs are seeded.
2. **DB schema** — an `items` table (owner, position
   inventory/equipment-slot/warehouse, durability, plus/gem sockets, etc.) and
   a `tq-db` model.
3. **Inventory lifecycle** — load on login → `MsgItemInfo` burst for
   `SendItems`; equip/unequip with stat application and `MsgPlayer` equipment
   broadcast; use/consume (potions); drop/pickup through `FloorItem` and a
   ground-item spawn packet.
4. **Money** — silver/CP mutations already persist on the character; drop
   money, bank operations, and trade need transactional handling.

See [Missing & Roadmap](../analysis/missing-and-roadmap.md) for suggested
ordering.
