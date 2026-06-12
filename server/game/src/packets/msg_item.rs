use super::{MsgTalk, TalkChannel};
use crate::state::State;
use crate::ActorState;
use async_trait::async_trait;
use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};

/// Enumeration type for defining item actions that may be requested by the
/// user, or given to by the server. Allows for action handling as a packet
/// subtype. Enums should be named by the action they provide to a system in the
/// context of the player item.
#[derive(Default, Debug, FromPrimitive, IntoPrimitive, Clone, Copy)]
#[repr(u32)]
enum ItemActionType {
    #[default]
    Unknown,
    Buy = 1,
    Sell = 2,
    Drop = 3,
    Use = 4,
    Equip = 5,
    Unequip = 6,
    SplitItem = 7,
    CombineItem = 8,
    QueryMoneySaved = 9,
    SaveMoney = 10,
    DrawMoney = 11,
    DropMoney = 12,
    SpendMoney = 13,
    Repair = 14,
    RepairAll = 15,
    Ident = 16,
    Durability = 17,
    DropEquipement = 18,
    Improve = 19,
    UpLevel = 20,
    BoothQuery = 21,
    BoothAdd = 22,
    BoothDel = 23,
    BoothBuy = 24,
    SynchroAmount = 25,
    Fireworks = 26,
    Ping = 27,
    Enchant = 28,
    BoothAddCPs = 29,
}

/// Message containing an item action command. Item actions are usually
/// performed to manage player equipment, inventory, money, or item shop
/// purchases and sales. It is serves a second purpose for measuring client
/// ping.
#[derive(Debug, Serialize, Deserialize, Clone, PacketID)]
#[packet(id = 1009)]
pub struct MsgItem {
    character_id: u32,
    param0: u32,
    action_type: u32,
    client_timestamp: u32,
    param1: u32,
}

#[async_trait]
impl PacketProcess for MsgItem {
    type ActorState = ActorState;
    type Error = crate::Error;
    type State = State;

    async fn process(&self, _state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        let action = self.action_type.into();
        match action {
            ItemActionType::Ping => {
                // Echo the packet back unmodified; the client computes the
                // displayed round-trip time from its own timestamp, so any
                // server-side adjustment only skews the result.
                actor.send(self.clone()).await?;
            },
            _ => {
                actor.send(self.clone()).await?;
                let p = MsgTalk::from_system(
                    self.character_id,
                    TalkChannel::Service,
                    format!("Missing Item Action Type {:?}", action),
                );
                tracing::warn!(
                    ?action,
                    param0 = %self.param0,
                    param1 = %self.param1,
                    action_id = self.action_type,
                    "Missing Item Action Type",
                );
                actor.send(p).await?;
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use futures::FutureExt;
    use tq_network::{Message, PacketDecode};

    use super::*;
    use crate::test_utils::*;

    #[tokio::test]
    async fn ping_echoes_client_timestamp_unmodified() -> Result<(), crate::Error> {
        with_test_env(tracing::Level::DEBUG, |state, _actors| {
            async move {
                let (tx, mut rx) = tokio::sync::mpsc::channel(1);
                let actor = Actor::<ActorState>::new(tx);
                let msg = MsgItem {
                    character_id: 1,
                    param0: 0,
                    action_type: ItemActionType::Ping.into(),
                    client_timestamp: 0xDEAD_BEEF,
                    param1: 0,
                };
                msg.process(&state, &actor).await?;
                let echoed = match rx.try_recv() {
                    Ok(Message::Packet(id, bytes)) => {
                        assert_eq!(id, MsgItem::PACKET_ID);
                        MsgItem::decode(&bytes)?
                    },
                    other => panic!("expected an echoed MsgItem, got {other:?}"),
                };
                assert_eq!(echoed.client_timestamp, msg.client_timestamp);
                assert_eq!(echoed.character_id, msg.character_id);
                assert_eq!(echoed.action_type, msg.action_type);
                Ok(())
            }
            .boxed()
        })
        .await
    }

    /// QA (SOC-7): boundary timestamps must round-trip bit-exact. The old
    /// `client_timestamp + 30` fudge would overflow-panic on `u32::MAX` in
    /// debug builds and skew every other value.
    #[tokio::test]
    async fn ping_echoes_boundary_timestamps_unmodified() -> Result<(), crate::Error> {
        with_test_env(tracing::Level::DEBUG, |state, _actors| {
            async move {
                for timestamp in [0u32, 1, 29, 30, u32::MAX - 30, u32::MAX] {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(8);
                    let actor = Actor::<ActorState>::new(tx);
                    let msg = MsgItem {
                        character_id: 1,
                        param0: 0,
                        action_type: ItemActionType::Ping.into(),
                        client_timestamp: timestamp,
                        param1: 0,
                    };
                    msg.process(&state, &actor).await?;
                    let echoed = match rx.try_recv() {
                        Ok(Message::Packet(id, bytes)) => {
                            assert_eq!(id, MsgItem::PACKET_ID);
                            MsgItem::decode(&bytes)?
                        },
                        other => panic!("expected an echoed MsgItem, got {other:?}"),
                    };
                    assert_eq!(echoed.client_timestamp, timestamp, "timestamp {timestamp} was modified");
                }
                Ok(())
            }
            .boxed()
        })
        .await
    }

    /// QA (SOC-7): the echo must carry the full 5017 field set (20-byte
    /// body: ID, Data, Action, SystemTime, Amount) verbatim, and a ping
    /// must produce exactly one outbound packet — no diagnostic chatter.
    #[tokio::test]
    async fn ping_echoes_full_field_set_and_nothing_else() -> Result<(), crate::Error> {
        with_test_env(tracing::Level::DEBUG, |state, _actors| {
            async move {
                let (tx, mut rx) = tokio::sync::mpsc::channel(8);
                let actor = Actor::<ActorState>::new(tx);
                let msg = MsgItem {
                    character_id: 0xAABB_CCDD,
                    param0: 0x1122_3344,
                    action_type: ItemActionType::Ping.into(),
                    client_timestamp: 0x5566_7788,
                    param1: 0x99AA_BBCC,
                };
                msg.process(&state, &actor).await?;
                let echoed = match rx.try_recv() {
                    Ok(Message::Packet(id, bytes)) => {
                        assert_eq!(id, MsgItem::PACKET_ID, "echo must be packet 1009");
                        assert_eq!(bytes.len(), 20, "5017 MsgItem body is five u32 fields");
                        MsgItem::decode(&bytes)?
                    },
                    other => panic!("expected an echoed MsgItem, got {other:?}"),
                };
                assert_eq!(echoed.character_id, msg.character_id);
                assert_eq!(echoed.param0, msg.param0);
                assert_eq!(echoed.action_type, msg.action_type);
                assert_eq!(echoed.client_timestamp, msg.client_timestamp);
                assert_eq!(echoed.param1, msg.param1);
                assert!(
                    rx.try_recv().is_err(),
                    "ping must not produce extra packets (e.g. the missing-action diagnostic)"
                );
                Ok(())
            }
            .boxed()
        })
        .await
    }

    /// QA (SOC-7 regression guard): a hostile/unknown action id must still
    /// take the missing-action arm — echo plus a `MsgTalk` (1004) service
    /// diagnostic — and must not be misrouted into the ping echo.
    #[tokio::test]
    async fn unknown_action_still_sends_missing_action_diagnostic() -> Result<(), crate::Error> {
        with_test_env(tracing::Level::DEBUG, |state, _actors| {
            async move {
                for hostile_action in [0u32, 7, 30, 0xDEAD_BEEF, u32::MAX] {
                    let (tx, mut rx) = tokio::sync::mpsc::channel(8);
                    let actor = Actor::<ActorState>::new(tx);
                    let msg = MsgItem {
                        character_id: 1,
                        param0: 0,
                        action_type: hostile_action,
                        client_timestamp: 0,
                        param1: 0,
                    };
                    msg.process(&state, &actor).await?;
                    match rx.try_recv() {
                        Ok(Message::Packet(id, _)) => {
                            assert_eq!(id, MsgItem::PACKET_ID, "first reply for action {hostile_action} must be the echo")
                        },
                        other => panic!("expected an echoed MsgItem for action {hostile_action}, got {other:?}"),
                    }
                    match rx.try_recv() {
                        Ok(Message::Packet(id, _)) => assert_eq!(
                            id,
                            MsgTalk::PACKET_ID,
                            "action {hostile_action} must produce the missing-action MsgTalk diagnostic"
                        ),
                        other => {
                            panic!("expected a MsgTalk diagnostic for action {hostile_action}, got {other:?}")
                        },
                    }
                }
                Ok(())
            }
            .boxed()
        })
        .await
    }

    /// QA (SOC-7 hostile input): 5017 MsgItem requests legitimately arrive
    /// in multiple sizes — a truncated ping body must fail decoding with a
    /// clean error, never a panic.
    #[test]
    fn truncated_msg_item_decode_errors_cleanly() {
        // Full body is 20 bytes; try every shorter length, including the
        // 4267-era 16-byte layout without the trailing Amount field.
        for len in 0..20usize {
            let bytes = bytes::Bytes::from(vec![0u8; len]);
            let res = MsgItem::decode(&bytes);
            assert!(res.is_err(), "decoding a {len}-byte MsgItem body must error, got {res:?}");
        }
        // Exactly 20 bytes must decode.
        let bytes = bytes::Bytes::from(vec![0u8; 20]);
        assert!(MsgItem::decode(&bytes).is_ok());
    }
}
