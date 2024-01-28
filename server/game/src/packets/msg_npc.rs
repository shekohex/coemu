use num_enum::{FromPrimitive, IntoPrimitive};
use serde::Deserialize;
use tq_network::{Actor, PacketID, PacketProcess};

use crate::entities::NpcKind;
use crate::packets::{MsgAction, MsgTalk, MsgTaskDialog};

#[derive(Default, Debug, Clone, Copy, FromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum NpcActionKind {
    #[default]
    Activate = 0,
    AddNpc = 1,
    LeaveMap = 2,
    DeleteNpc = 3,
    ChangePosition = 4,
    LayNpc = 5,
    CancelInteraction = 255,
}

/// This packet is used to interact with a NPC and contains multiple
/// DialogAction types that are used to determine the type of interaction.
#[derive(Debug, Deserialize, Clone, PacketID)]
#[packet(id = 2031)]
pub struct MsgNpc {
    npc_id: u32,
    data: u32,
    action: u16,
    kind: u16,
}

#[async_trait::async_trait]
impl PacketProcess for MsgNpc {
    type ActorState = crate::ActorState;
    type Error = crate::Error;
    type State = crate::State;

    async fn process(&self, state: &Self::State, actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        tracing::debug!(
            npc_id = self.npc_id,
            data = self.data,
            action = ?NpcActionKind::from(self.action),
            kind = ?NpcKind::from(self.kind as u8),
            "MsgNpc received"
        );
        let me = actor.entity();
        let mycharacter = me.as_character().ok_or(crate::Error::CharacterNotFound)?;
        let mymap = state.try_map(me.basic().map_id())?;
        let npc = match mymap.npc(self.npc_id) {
            Some(npc) => npc,
            None => {
                actor
                    .send(MsgTalk::from_system(
                        me.id(),
                        super::TalkChannel::System,
                        format!("NPC {} not found", self.npc_id),
                    ))
                    .await?;
                return Ok(());
            },
        };
        let my_loc = me.basic().location();
        let npc_loc = npc.entity().location();
        let in_screen = tq_math::in_screen(my_loc.into(), npc_loc.into());
        if !in_screen {
            // TODO: Player is using some kind of hack to interact with NPCs
            // that are not in the screen.
            return Ok(());
        }
        // Storage NPCs
        if npc.is_storage() {
            actor
                .send(MsgAction::from_character(
                    mycharacter,
                    4, // WHAT IS THIS?
                    super::ActionType::OpenDialog,
                ))
                .await?;
            return Ok(());
        }
        let npc_id = npc.id();
        let npc_name = npc.entity().name();
        // For now, lets try sending a dummy dialog
        let dialog = MsgTaskDialog::builder()
            .text(format!("Hello, My name is {npc_name} and my id is {npc_id}",))
            .with_edit(1, "What is your name?")
            .with_option(255, "Nice to meet you")
            .and()
            .with_avatar(47)
            .build();
        actor.send_all(dialog).await?;
        Ok(())
    }
}
