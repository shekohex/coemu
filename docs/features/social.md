# Social Systems (Trade, Team, Friends, Guilds, Marriage) — ❌ Missing

None of the social systems exist. Current footprint in code:

| System | What exists | What's missing |
|---|---|---|
| **Friends** | `ActionType::SendAssociates` echo stub with `TODO: send MsgFriend` ([msg_action.rs:359](../../server/game/src/packets/msg_action.rs#L359)); `TalkChannel::Friend` | `MsgFriend` packet, friends table, online notifications, whisper integration |
| **Team / party** | `ActionType::QueryTeamMember` enumerated, unhandled; `TalkChannel::Team` | `MsgTeam` / `MsgTeamMember` packets, invites/kick/leave, XP sharing, team kill mode |
| **Guilds (syndicates)** | `ActionType::ConfirmGuild` echo stub with `TODO: send MsgSyndicateAttributeInfo` ([msg_action.rs:377](../../server/game/src/packets/msg_action.rs#L377)); `TalkChannel::Guild`; `NpcKind::SynFlag`/`SynTrans` | `MsgSyndicate` + attribute packets, guild tables (members, ranks, funds), guild war, syn flags on maps |
| **Trade** | Nothing | `MsgTrade` packet, trade-window session state, atomic item+money swap (depends on items) |
| **Marriage** | `TalkChannel::Spouse` enum value only | Court/marry via `MsgInteract`, spouse name on `MsgPlayer` |
| **Whisper** | Channel enumerated | Name-based routing through a global online-player registry (the `State` entity registry in [state/mod.rs](../../server/game/src/state/mod.rs) is a starting point) |

## Dependencies

Trade and guild funds depend on the item/money systems
([items-and-inventory.md](../features/items-and-inventory.md)); team XP
sharing depends on combat/progression. Friends and whisper are implementable
today with only a new table and packet — they are the cheapest social wins.
