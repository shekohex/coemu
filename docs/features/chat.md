# Chat — 🟡 Partial

## What works

`MsgTalk` (1004, [msg_talk.rs](../../server/game/src/packets/msg_talk.rs)):

- All 5017 chat channels and styles are enumerated (`TalkChannel`,
  `TalkStyle`), and `MsgTalk` doubles as the login/registration status channel
  (`ANSWER_OK`, `NEW_ROLE`, rejection messages).
- Incoming chat is broadcast to the sender's **map region**
  ([msg_talk.rs:143](../../server/game/src/packets/msg_talk.rs#L143)).
- Messages starting with `$` are parsed as GM commands
  (see [gm-commands.md](../features/gm-commands.md)).
- System messages are used heavily by other handlers (kill-mode notices,
  "Invalid Location", missing-handler diagnostics).

## What's missing

The handler has an explicit `TODO: Implement this properly`
([msg_talk.rs:144](../../server/game/src/packets/msg_talk.rs#L144)). Channel
semantics are ignored — everything is region-local:

- **Whisper** (private messages by recipient name)
- **World / Broadcast / Announce** (server-wide, usually CP-gated)
- **Team / Guild / Friend / Spouse** channels (the underlying systems don't
  exist)
- Chat moderation: no mute, no flood control, no message length policing
  beyond the wire format.

## Ping

`MsgItem::Ping` echoes the client timestamp (+30 ms fudge) so the client can
display latency ([msg_item.rs:72](../../server/game/src/packets/msg_item.rs#L72)).
The code comments admit the offset trick produces negative pings and should be
reworked.
