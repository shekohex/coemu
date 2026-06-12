# Spirited's Conquer Online Documentation — Research Notes (Patch 5017)

> Researched 2026-06-11. "Spirited" (Gareth Jensen) maintains the most useful
> public documentation ecosystem for Conquer Online server development. These
> notes map that ecosystem and capture everything relevant to a 5017 emulator.
> Distilled packet/data details live in
> [co-5017-protocol-notes.md](co-5017-protocol-notes.md).

## Source map

| Resource | What it is | Value for coemu |
|---|---|---|
| [Conquer Online Development Wiki](https://conquer-online.github.io/wiki/) ([repo](https://github.com/conquer-online/wiki), formerly [GitLab](https://gitlab.com/conquer-online/wiki/-/wikis/home)) | Community wiki Spirited's guides point to; mdBook with 230+ documented packets, algorithms, constants, file formats, security | **Primary reference.** Many packets have explicit per-patch sections including 5017 |
| [Comet](https://spirited.io/project/comet/) ([GitLab](https://gitlab.com/spirited/comet), [GitHub mirror](https://github.com/conquer-online/comet)) | Spirited's instructional .NET server; one git branch per patch: 4274, 4294, 4343, **5017**, 5065, 5187 | Reference implementation of the 5017 login pipeline; see scope note below |
| [cooldown.dev](https://cooldown.dev/) | Spirited's CO development forum (successor to elitepvpers threads) | Patch-specific threads, e.g. [5017 login sequence](https://cooldown.dev/topic/41-5017-login-sequence/), client downloads, packet RE discussions |
| ConquerWiki (conquerwiki.com) | Older DokuWiki packet reference | Timed out during research; superseded by the GitHub wiki above |

## What patch 5017 is

Per Spirited's Comet project page, 5017 is the late-2007-era client. Relative
to earlier patches it adds:

- The **pay-to-win shopping mall** (CP/EMoney purchases — first patch where
  Conquer Points appear throughout the protocol: trade, booths, user attributes).
- **+12 items** (extended composition/plus ratings).
- **WuXing Oven** (item composition/improvement UI).
- New fonts.

The next protocol-relevant change is in 5018: **the DH key exchange was added
in 5018 — 5017 does *not* use it** (per Spirited on the
[5017 login sequence thread](https://cooldown.dev/topic/41-5017-login-sequence/)).
This matches coemu's current cipher suite (TQ/CQ/RC5, no DH) — correct for 5017.

## 5017 login sequence (cooldown.dev thread)

1. Client connects to the **account server**, sends `MsgAccount` (1051) with
   credentials.
2. Account server authenticates and replies with token + game server IP/port
   (`MsgConnectEx` / 1055 in coemu terms; thread also references a 1052
   response type).
3. Client connects to the **game server** and sends `MsgConnect` with the token.
4. Game server validates the token and proceeds with the login burst
   (character info or "new role" prompt → `MsgRegister` flow).

Implementation pitfall documented in the thread: a race where the socket
begins receiving before per-client state is initialized causes intermittent
hangs at "Logging into the server…". The fix is ordering: initialize client
state fully *before* starting the receive loop. coemu's actor model already
serializes this, but it's worth a regression test once the game server is
wasmified (handler registration must complete before the socket pumps packets).

## Comet 5017 branch — scope reality check

Inspected via the GitLab API (`spirited/comet`, branch `5017`). The game
server contains only: `MsgAction`, `MsgConnect`, `MsgItem`, `MsgRegister`,
`MsgTalk`, `MsgUserInfo` packet handlers, a `DbCharacter` model, and
login/creation state. Database is MySQL/MariaDB via Entity Framework.

**Conclusion: Comet 5017 is a login-workflow skeleton — coemu is already
ahead of it** (movement, portals, screens, NPCs, weather are not in Comet).
Comet is useful as a second opinion on login/creation packet handling, not as
a gameplay reference. For gameplay mechanics, the wiki + the feature gaps in
[the feature matrix](../features/README.md) are the guide.

## Wiki coverage assessment for 5017

Checked page-by-page (raw markdown from `conquer-online/wiki`):

**Strong, with explicit 5017 sections:**
- `MsgItem` (1009) — full 5017 action table (29 actions incl. ENCHANT and
  BOOTH_ADD_EMONEY, both new in 5017)
- `MsgItemInfo` (1008) — 5017 layout (36 bytes, socket progress field)
- `MsgInteract` (1022) — 5017 layout (Progress field) + magic-attack
  obfuscation macros (bit rotations/XOR, from the leaked Soul source)
- `MsgUserAttrib` (1017) — 5017 attribute list (adds EMONEY, LUCKY_SECONDS,
  OFFLINE_TRAINING) + full status-flag bitmaps
- `MsgTrade` (1056) — 5017 adds CP/EMoney trade actions (12–14)

**Documented at 4267 and stable through 5017** (5000-series layouts match or
have noted diffs): `MsgTeam` (1023), `MsgTeamMember` (1026), `MsgSyndicate`
(1107), `MsgMagicInfo` (1103), `MsgMagicEffect` (1105), `MsgMapItem` (1101),
`MsgNpcInfo` (2030), `MsgWeaponSkill` (1025), `MsgAllot` (1024).

**Stub pages (title only, no content yet)** — do not rely on the wiki for
these; use leaked-source references (Soul/COPSv6) or RE instead:
- `MsgTaskDialog` (NPC dialog) — coemu already has a working implementation
- PK Mode constants
- Damage calculation, Attributes calculation
- Item composition rates
- `magictype.dat`, `monstertype.dat` file formats

**Useful adjacent pages:** identifier ranges (entity ID partitioning),
`itemtype.dat` format (59 fields, documented for 5517 but structurally
informative), `levelexp.dat` (XOR key documented, with parser script),
DMap/DAT/TME file formats, string packer, timestamps.

## Takeaways for coemu

1. The wiki's per-patch packet tables are authoritative enough to implement
   items, combat interaction, attribute sync, trade, team, and guild packets
   for 5017 without client RE.
2. Where the wiki is a stub (damage formulas, PK constants, magictype format),
   plan for reverse-engineering or cross-referencing other open-source 5017
   servers (COPSv6 is the wiki's own cited observation source for 5017).
3. 5017's protocol marks the EMoney/CP introduction — schema for items,
   trade, and booths should include CP amounts from day one.
4. Several wiki pages carry explicit anti-cheat warnings (MsgAllot spoofing,
   MsgItem item/fuel validation, identifier predictability). These are
   recorded as acceptance criteria in the [user stories](../user-story/).
