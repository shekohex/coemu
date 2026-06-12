# Epic: Social & Communication

> Source: [social.md](../features/social.md), [chat.md](../features/chat.md)
> (🟡 partial), [MsgTeam/MsgSyndicate/MsgInteract notes](../notes/co-5017-protocol-notes.md).
> SOC-1/SOC-2 have no dependencies and are the cheapest player-visible wins.

## SOC-1: Whisper

**As a** player **I want** to message a player by name anywhere on the
server **so that** I can coordinate privately.

**Acceptance criteria:**
- `MsgTalk` whisper channel routes by recipient name through a global
  online-player registry (extending the `State` entity registry); offline
  recipient produces the standard system notice.
- Resolves the `TODO` at [msg_talk.rs:144](../../server/game/src/packets/msg_talk.rs#L144)
  for the whisper case; flood control added (rate limit per sender).

## SOC-2: Friends list

**As a** player **I want** a friends list with online notifications
**so that** I can keep track of people I play with.

**Acceptance criteria:**
- `friends` table (mutual pairs); `ActionType::SendAssociates` stub
  ([msg_action.rs:359](../../server/game/src/packets/msg_action.rs#L359)) replaced
  with real `MsgFriend` bursts at login.
- Add (mutual consent), remove, online/offline notifications to friends;
  `TalkChannel::Friend` routes to online friends.

## SOC-3: World and broadcast chat

**As a** player **I want** world chat **so that** the server has a public
channel.

**Acceptance criteria:**
- World channel broadcasts to all online players with a per-player cooldown;
  GM announce channel restricted by privilege; remaining `MsgTalk` channel
  semantics stop falling through to region-local.

## SOC-4: Teams

**As a** player **I want** parties of up to five **so that** we can hunt
together and share XP.

**Acceptance criteria:**
- `MsgTeam` (1023) actions implemented per the
  [protocol notes](../notes/co-5017-protocol-notes.md#msgteam-1023--msgteammember-1026):
  create, invite/apply + accept (consent both ways), leave, kick (leader),
  dismiss, open/close join, money/item pickup permissions.
- `MsgTeamMember` (1026) maintains the member HP sidebar (adds on join, drops
  on leave; HP refreshes batched with `MsgUserAttrib` life updates).
- Team leader status flag (USERSTATUS_TEAM_LEADER); `TalkChannel::Team`
  routes to members; XP sharing rules apply on kills (MON-4).
- Teams are in-memory session state only — no persistence.

## SOC-5: Guilds (syndicates)

**As a** player **I want** to create and run a guild **so that** large-group
identity, funds, and wars are possible.

**Acceptance criteria:**
- Guild tables: guild (name unique, leader, fund, bulletin), members (rank,
  donation); creation via the guild NPC with the 5017 silver cost.
- `MsgSyndicate` (1107) actions: apply/invite + accept, leave, kick,
  ally/enemy declare+clear, DONATE_MONEY (fund transaction), QUERY_SYN_NAME
  (`MsgName`), QUERY_SYNATTR (`MsgSyndicateAttributeInfo` — resolves the
  `ConfirmGuild` TODO at [msg_action.rs:377](../../server/game/src/packets/msg_action.rs#L377)),
  SET_SYN push at login, DESTROY_SYN (leader only).
- Guild name shows on spawned players; `TalkChannel::Guild` routes to online
  members; ranks gate kick/ally/disband actions server-side.

## SOC-6: Marriage

**As a** player **I want** to propose and marry **so that** the spouse
mechanics work.

**Acceptance criteria:**
- `MsgInteract` COURT (8) / MARRY (9) flow with consent; spouse name persists
  and appears in `MsgPlayer` spawns; `TalkChannel::Spouse` routes to the
  spouse only.

## SOC-7: Accurate ping

**As a** player **I want** the latency display to be truthful **so that** I
can judge my connection.

**Acceptance criteria:**
- `MsgItem::Ping` echo reworked to drop the +30 ms fudge (code admits it
  produces negative pings at
  [msg_item.rs:72](../../server/game/src/packets/msg_item.rs#L72)); echo the
  client timestamp unmodified.
