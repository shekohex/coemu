# Epic: Economy & Trade

> Source: [economy.md](../features/economy.md) (❌ missing), social.md (trade),
> [MsgItem/MsgTrade notes](../notes/co-5017-protocol-notes.md). Everything
> here is downstream of [Items & Inventory](items-and-inventory.md). All
> value transfers are ACID transactions per
> [tech-stack-best-practices.md](../analysis/tech-stack-best-practices.md).

## ECO-1: Buy from and sell to NPC shops

**As a** player **I want** to buy and sell items at shopkeeper NPCs
**so that** I can gear up and convert loot to silver.

**Acceptance criteria:**
- Shop inventory data (`shop.dat` equivalent: NPC id → item types) is seeded
  into a table and loaded at boot.
- `MsgItem` BUY (1) validates: NPC is a shopkeeper, sells that item type,
  player is within interaction range of the NPC, has funds and inventory
  space; 5017's `Amount` field supports multi-buy.
- SELL (2) pays the catalogue sell price (fraction of `price`); money updates
  push via `MsgUserAttrib` MONEY; item grant/removal and payment are atomic.
- CP-priced (EMoney) mall items charge CP, not silver — 5017's defining
  feature.

## ECO-2: Warehouse storage

**As a** player **I want** per-city warehouses **so that** I can store items
beyond my 40 inventory slots.

**Acceptance criteria:**
- Interacting with a storage NPC (today a bare `OpenDialog` at
  [msg_npc.rs:71](../../server/game/src/packets/msg_npc.rs#L71)) lists stored
  items via `MsgItemInfo` with storage positions (201+).
- Deposit/withdraw move items between inventory and that NPC's warehouse,
  validating proximity and capacity; contents persist per character per
  warehouse.

## ECO-3: Banking

**As a** player **I want** to deposit silver at the bank **so that** money
not on hand is safe from PK drops.

**Acceptance criteria:**
- QUERY_MONEY (9) / SAVE_MONEY (10) / DRAW_MONEY (11) implemented against a
  new `money_saved` column; deposits/withdrawals atomic with on-hand money.
- Caps enforced (on-hand and saved are u32 in the protocol); withdraw of more
  than saved is rejected.

## ECO-4: Player-to-player trade

**As a** player **I want** a trade window with another player **so that** we
can exchange items, silver, and CPs safely.

**Acceptance criteria:**
- Full `MsgTrade` (1056) flow per the
  [5017 action table](../notes/co-5017-protocol-notes.md#msgtrade-1056--player-trading-5017):
  APPLY → OPEN, ADDITEM (server answers `MsgItemInfo` action=TRADE),
  ADDMONEY, and 5017's CP actions ADDEMONEY/totals; both sides OK → SUCCESS.
- The swap commits **atomically**: items + silver + CPs change owners in one
  transaction or not at all (disconnects/cancel mid-trade leave both sides
  untouched).
- Validations: both online, same map and near each other, receiving side has
  inventory space, items are tradable (monopoly flag), offered amounts within
  current funds; offers reset if either side modifies the window after an OK.
- Trade-window sessions live in memory keyed by both heroes; either player
  moving/teleporting/disconnecting cancels via QUIT/FALSE.

## ECO-5: Player market booths

**As a** player **I want** to open a booth in the market **so that** I can
sell items while away from keyboard.

**Acceptance criteria:**
- CreateBooth (`MsgAction`) restricted to market booth tiles; BOOTH_ADD (22)
  lists an item for silver, BOOTH_ADD_EMONEY (29, new in 5017) for CPs;
  BOOTH_DEL delists; BOOTH_QUERY shows listings to browsers.
- BOOTH_BUY transfers item and payment atomically between the two players,
  both potentially online; ⚠ proximity validation per the wiki warning (no
  vending at a distance).
- `LeaveBooth` (already partially handled at
  [msg_action.rs:182](../../server/game/src/packets/msg_action.rs#L182)) tears the
  booth down; disconnect does the same.

## ECO-6: Money on the ground

**As a** player **I want** dropped silver to appear and be lootable
**so that** money drops from PK deaths and generosity work.

**Acceptance criteria:**
- DROP_MONEY creates a floor item with the correct money mesh tiers (silver
  pile looks vary by amount); pickup adds to on-hand money atomically with
  floor-item deletion.

## ECO-7: Anti-abuse guardrails for the economy

**As an** operator **I want** all economy actions rate-limited and audited
**so that** dupes and bots are detectable and bounded.

**Acceptance criteria:**
- Per-connection rate limits on BUY/SELL/trade/booth actions.
- An append-only audit log (structured `tracing` events at minimum) for every
  value transfer: who, what, amounts, before/after balances.
- Negative/overflow amounts rejected at deserialization (u32 arithmetic
  checked on all money paths).
