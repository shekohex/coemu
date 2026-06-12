# Conquer Online 5017 — Protocol & Data Notes

> Distilled 2026-06-11 from the [Conquer Online Development Wiki](https://conquer-online.github.io/wiki/)
> (raw markdown, `conquer-online/wiki`), selecting the patch-5017 sections
> where they exist. Companion to
> [spirited-conquer-docs-5017.md](spirited-conquer-docs-5017.md). Everything
> here targets features coemu has not implemented yet (see
> [feature matrix](../features/README.md)).

## Entity identifier ranges

The game server statically partitions 32-bit role IDs. Sticking to these
ranges matters because the client infers role *kind* from the ID:

| Range | Kind |
|---|---|
| 1 – 99,999 | System (static) NPC |
| 100,001 – 199,999 | Dynamic NPC |
| 400,001 – 499,999 | Monster (world spawn) |
| 500,001 – 599,999 | Pet (syndicate spawn) |
| 700,001 – 799,999 | Call pet (player-summoned) |
| 900,001 – 989,999 | Magic trap (player) |
| 990,001 – 999,999 | System trap |
| 1,000,000 – 3,999,999,999 | Hero (player character) |

Monster IDs must be recycled (reserve blocks per spawn area). Wiki warning:
incremental account/item IDs are predictable — consider an LFSR-style
obfuscated counter for item IDs.

## MsgItem (1009) — item actions, 5017 layout

24 bytes: `ID:u32`, `Data:u32`, `Action:u32`, `SystemTime:u32`, `Amount:u32`
(Amount is new in 5017 vs 4267's 20-byte layout). Requests may arrive
truncated — don't read unspecified trailing fields.

5017 action table (Server = client request, Client = server push):

| Val | Action | Dir | ID / Data semantics |
|---|---|---|---|
| 1 | BUY | →Server | NPC id / item type |
| 2 | SELL | →Server | NPC id / item id |
| 3 | DROP | →Server | item id / X(low),Y(high) |
| 4 | EQUIP | →Server | item id / position |
| 5 | UPDATE | →Client | item id / position |
| 6 | UNEQUIP | →Client | item id / position |
| 8 | COMBINE | →Server | item id / fuel item id |
| 9 | QUERY_MONEY | →Server | NPC id / money (resp) |
| 10 | SAVE_MONEY | →Server | NPC id / money |
| 11 | DRAW_MONEY | →Server | NPC id / money |
| 12 | DROP_MONEY | →Server | money / X(low),Y(high) |
| 14 | REPAIR | →Server | item id |
| 17 | DURABILITY | →Client | position / durability |
| 18 | DROP_EQUIPMENT | →Client | item id / position |
| 19 | IMPROVE | →Server | item id / dragonball item id |
| 20 | UPLEV | →Server | item id / meteor item id |
| 21 | BOOTH_QUERY | →Server | item id |
| 22 | BOOTH_ADD | →Server | item id / price (silver) |
| 23 | BOOTH_DEL | →Server | item id / shop id |
| 24 | BOOTH_BUY | →Server | item id / shop id |
| 27 | PING | →Server | player id (already handled in coemu) |
| 28 | ENCHANT *(new 5017)* | →Server | item id / gem item id |
| 29 | BOOTH_ADD_EMONEY *(new 5017)* | →Server | item id / CP price |

Wiki anti-dupe warnings: validate that improve/uplev fuel item type matches
expectations and differs from the target item; validate proximity (can't
upgrade/vend at a distance or cross-map); responses to "show item" should use
a fresh share id, not the item's real unique id.

## MsgItemInfo (1008) — item description, 5017 layout

36 bytes (❓ wiki marks 5017 layout unverified):
`ItemID:u32`, `ItemType:u32`, `Amount:u16` (durability), `MaxAmount:u16`,
`Action:u8`, `Status:u8`, `Position:u8`, pad, `SocketProgress:u32`,
`Socket1:u8`, `Socket2:u8`, `Magic1:u8` (rebirth effect), `Magic2:u8`,
`Magic3:u8` (plus rating), `Bless:u8`, `Enchant:u8`, pad, `Data:u32`.

- Action: 0 none · 1 ADD_ITEM · 2 TRADE · 3 UPDATE · 4 OTHER_PLAYER_EQUIPMENT
  · 5 AUCTION.
- Status bits: 0x01 unidentified · 0x02 cannot-repair · 0x04 fixed-durability
  · 0x08 magic-add.
- Positions: 0 inventory · 1 helmet · 2 necklace · 3 armor · 4 weapon-R ·
  5 weapon-L · 6 ring · 7 treasure(bottle) · 8 boots · 201 storage ·
  202 trunk · 203 treasure bag · 254 ground · 255 none.

This drives the login `SendItems` burst, equip/unequip, trade window item
display, and warehouse listings.

## MsgMapItem (1101) — floor items & traps (4267 layout, pre-5095)

18 bytes: `ID:u32` (map-item id from trap/item ranges), `Type:u32` (item type
or trap look), `X:u16`, `Y:u16`, `Action:u16`.
Actions: 1 CREATE (→client) · 2 DELETE (→client) · 3 PICK (→server request)
· 10 CAST_TRAP · 11 SYNCHRO_TRAP · 12 DROP_TRAP.

## MsgInteract (1022) — combat & social interactions, 5017 layout

32 bytes: `SystemTime:u32`, `Sender:u32`, `Target:u32`, `X:u16`, `Y:u16`,
`Type:u32`, `Data:u32`, `Progress:u32` (Progress is new vs 4267).

| Val | Type | Dir | Data |
|---|---|---|---|
| 2 | ATTACK (physical) | →Server | damage (in response) |
| 8 | COURT (propose) | →Server | |
| 9 | MARRY (accept) | →Server | |
| 14 | KILL (confirm kill) | →Client | count |
| 21 | MAGIC_ATTACK | →Server | high word: magic type |
| 23 | REFLECT_WEAPON | →Client | damage |
| 24 | BUMP (dash knockback) | →Client | direction |
| 25 | SHOOT (archery) | →Server | damage (resp) |
| 26 | REFLECT_MAGIC | →Client | damage |
| 30 | JAR_PROGRESS (Cloud Saint Jar) | →Server | progress=count (resp) |

**Magic-attack obfuscation**: in MAGIC_ATTACK requests the client encodes
type/target/x/y with bit-rotations and XORs keyed on the sender ID (macros
from the Soul source, reproduced on the wiki page):

```text
type   = ror16(raw_type   - 0x14BE, 3)        ^ sender ^ 0x915D
target = ror32(raw_target - 0x8B90B51A ^ sender ^ 0x5F2D2463, 32-13)   // see wiki for exact order
x      = ror16(raw_x      - 0xDD12, 1)        ^ sender ^ 0x2ED6
y      = ror16(raw_y      - 0x76DE, 5)        ^ sender ^ 0xB99B
```

Decode is the inverse (XOR first, rotate back, add constant). Exact macro
text: [wiki MsgInteract page](https://conquer-online.github.io/wiki/network/messages/msginteract.html).
Note: abort-magic exists both here and duplicated in `MsgAction`.

## MsgMagicInfo (1103) / MsgMagicEffect (1105) / MsgWeaponSkill (1025)

- **MsgMagicInfo** (12 bytes): `Exp:u32`, `Type:u16`, `Level:u16` — one per
  learned spell in the login burst. Spell catalogue comes from
  `magictype.dat` (format page is a wiki stub).
- **MsgMagicEffect** (36+ bytes): `User:u32`, `Target:u32` (or X/Y packed),
  `Type:u16`, `Level:u16`, `EffectNum:u32`, then repeated RoleInfo
  `{Role:u32, Damage:u32, Reserved:u32}` — broadcasts a cast and per-target
  damage for AoE. In later patches coordinates are obfuscated like
  MsgInteract; base 4267 layout applies at 5017.
- **MsgWeaponSkill** (16 bytes): `Type:u32`, `Level:u32`, `Exp:u32` —
  proficiency updates; data files `weaponskillname.ini` /
  `weaponskilllevelexp.ini`.

## MsgUserAttrib (1017) — attribute sync, 5017

Header: `HeroID:u32`, `AttributeNum:u32`, then repeated
`{AttribType:u32, Data:u32[2]}`. 5017 attribute types:

```
0 LIFE        1 MAXLIFE      2 MANA          3 MAXMANA
4 MONEY       5 EXP          6 PK            7 PROFESSION
8 SIZE_ADD    9 PP          11 ADDPOINT     12 LOOK
13 LEV       14 SOUL        15 HEALTH       16 FORCE
17 SPEED     18 BLESS_SECONDS  19 DOUBLE_XP_SECONDS  20 SYN_WAR_POLE
21 CURSE_SECONDS  22 TIME_ADD_SECONDS  23 METEMPSYCHOSIS
26 USERSTATUS    27 HAIR
29 LUCKY_SECONDS (new)   30 EMONEY (new)   31 XP (moved from 28)
32 OFFLINE_TRAINING_PROGRESS (new)
```

USERSTATUS bitmap (5017): 0x01 flashing-name · 0x02 poisoned · 0x04
invisible · 0x10 xp-full · 0x40 team-leader · 0x80 adjust-dodge · 0x100
shield · 0x200 stigma · 0x400 ghost · 0x800 disappearing · 0x4000 red-name ·
0x8000 black-name · 0x40000 superman · 0x800000 cyclone · 0x4000000 dodge ·
0x8000000 fly · 0x40000000 cast-pray · 0x80000000 praying.

This is the workhorse packet for HP/MP/exp/PK-point/status updates — needed
by combat, progression, and PK stories alike.

## MsgAllot (1024) — stat point allocation

8 bytes: `Force:u8`, `Speed:u8`, `Health:u8`, `Soul:u8`. **Wiki warning:**
the client validates available points but a modified client can spoof any
values — the server must re-validate against the character's unallocated
points before applying.

## MsgTeam (1023) / MsgTeamMember (1026)

- **MsgTeam** (8 bytes): `Action:u32`, `Player:u32`. Actions: 0 CREATE ·
  1 APPLY_JOIN · 2 LEAVE · 3 ACCEPT_INVITE · 4 INVITE · 5 ACCEPT_JOIN ·
  6 DISMISS · 7 KICK_OUT · 8 CLOSE_TEAM · 9 OPEN_TEAM · 10/11
  CLOSE/OPEN_MONEY_ACCESS · 12/13 CLOSE/OPEN_ITEM_ACCESS.
- **MsgTeamMember**: `Action:u8` (0 add, 1 drop), `Amount:u8`, repeated
  `{Name:char[16], ID:u32, Lookface:u32, MaxHP:u16, HP:u16}` — drives the
  client's right-side member HP list. A message must be all-adds or
  all-removals.

## MsgSyndicate (1107) — guild actions (4267 layout, applies at 5017)

12 bytes: `Action:u32`, `Target:u32`. Actions: 1 APPLY_JOIN · 2 INVITE_JOIN ·
3 LEAVE_SYN · 4 KICKOUT_MEMBER · 6 QUERY_SYN_NAME (→MsgName) · 7 ALLY_APPLY ·
8 CLEAR_ALLY · 9 ANTAGONIZE · 10 CLEAR_ANTAGONIZE · 11 DONATE_MONEY ·
12 QUERY_SYNATTR (→MsgSyndicateAttributeInfo) · 14 SET_SYN (login push) ·
19 DESTROY_SYN.

## MsgTrade (1056) — player trading, 5017

12 bytes: `Data:u32`, `Action:u16`. 5017 extends 4267 with CP actions:

| Val | Action | Dir |
|---|---|---|
| 1 | APPLY (request trade) | both |
| 2 | QUIT | →Server |
| 3 | OPEN (open window) | →Client |
| 4 | SUCCESS / 5 FALSE | →Client |
| 6 | ADDITEM | →Server (server answers with MsgItemInfo action=TRADE) |
| 7 | ADDMONEY · 8 PLAYERTOTALMONEY · 9 HEROTOTALMONEY | mixed |
| 10 | OK (confirm) | both |
| 11 | ADDITEM_FALSE | →Client |
| 12 | PLAYERTOTALEMONEY · 13 ADDEMONEY · 14 HEROTOTALEMONEY *(new 5017 — CP trading)* | mixed |

(5022 later adds SUSPICIOUS_PROMPT/OK — not needed for 5017.)

## MsgNpcInfo (2030) — NPC spawn (4267 layout at 5017)

`ID:u32`, `X:u16`, `Y:u16`, `Role:u16`, `Lookface:u16`, `Sort:u16`, optional
packed name string. `Lookface = Type*10 + Direction`; type/name from
`npc.ini`. Sort: 0 dialog NPC · 1 task/window NPC · 2 dynamically-relocated.

## Game data files

| File | Status on wiki | Notes |
|---|---|---|
| `itemtype.dat` | Documented (5517 record layout, 59 `@@`-separated fields) | TQ file cipher, seed 9527. Field order differs by patch but the core (id, name, req\_\*, price, attack/defense, durability, gems, plus) is stable. Includes a Python→CSV parser on the page |
| `levelexp.dat` | Documented | Flat u32 array of per-level EXP, XOR-encrypted with a fixed 27-byte key (documented + parser script). Early patches; later superseded by `levexp.dat` |
| `magictype.dat` | **Stub** | Spell catalogue (levels, MP cost, power, range, exp milestones) — needed for skills; plan to RE or pull from another 5017 server's SQL conversion |
| `monstertype.dat` | **Stub** | Monster stats — same plan as magictype |
| `gamemap.dat`, DMap, TME | Documented | coemu already parses DMaps |

## Stubs / gaps to source elsewhere

Damage formulas, attribute (stat-per-level) tables, PK-mode constants, item
composition (+N) rates, and `MsgTaskDialog` are stub pages on the wiki as of
2026-06-11. The wiki's 5017 observations cite **COPSv6** and the leaked
**Soul** source — those are the fallback references when implementing combat
math and PK rules.
