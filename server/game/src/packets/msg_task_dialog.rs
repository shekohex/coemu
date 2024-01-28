use num_enum::{FromPrimitive, IntoPrimitive};
use serde::{Deserialize, Serialize};
use tq_network::{Actor, PacketID, PacketProcess};
use tq_serde::StringList;

use crate::constants;

#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum DialogActionKind {
    #[default]
    Unknown = 0,
    Text = 1,
    Link = 2,
    Edit = 3,
    Avatar = 4,
    ListLine = 5,
    Create = 100,
    Answer = 101,
    TaskId = 102,
}

/// This packet is used to interact with a NPC and contains multiple
/// DialogAction types that are used to determine the type of interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PacketID)]
#[packet(id = 2032)]
pub struct MsgTaskDialog {
    task_id: u32,
    avatar: u16,
    option_id: u8,
    action: u8,
    msgs: StringList,
}

impl MsgTaskDialog {
    pub fn builder() -> MultiTaskDialogBuilder<AddingText> {
        MultiTaskDialogBuilder::default()
    }
}

/// A Multi Task Dialog builder helps crafting a dialog with multiple options
/// and actions by building a list of [`MsgTaskDialog`] packets in very
/// chainable and ergonomic way.
#[derive(Debug, Clone, Default)]
pub struct MultiTaskDialogBuilder<S> {
    /// Holds the list of tasks that will be sent to the client.
    tasks: Vec<MsgTaskDialog>,
    /// Holds the current state.
    _state: std::marker::PhantomData<S>,
}

// Tasks are sent in a specific order, so we have different states to
// ensure that the builder is used correctly.
#[derive(Debug, Clone, Default)]
pub struct AddingText;
#[derive(Debug, Clone)]
pub struct AddingOptionOrEdit;
#[derive(Debug, Clone)]
pub struct AddingAvatar;
#[derive(Debug, Clone)]
pub struct Ready;

impl MultiTaskDialogBuilder<Ready> {
    /// Builds the list of tasks and returns it.
    pub fn build(mut self) -> Vec<MsgTaskDialog> {
        // Push the last task
        self.tasks.push(MsgTaskDialog {
            task_id: 0,
            avatar: 0,
            option_id: u8::MAX,
            action: DialogActionKind::Create.into(),
            msgs: Default::default(),
        });
        self.tasks
    }
}

impl MultiTaskDialogBuilder<AddingText> {
    /// Adds a text task to the list of tasks.
    ///
    /// # Arguments
    ///
    /// * `text` - A string slice representing the text to be added to the task.
    pub fn text<T: AsRef<str>>(mut self, text: T) -> MultiTaskDialogBuilder<AddingOptionOrEdit> {
        let text = text.as_ref();
        let msgs = text
            .as_bytes()
            .chunks(constants::MAX_TXT_LEN)
            .flat_map(std::str::from_utf8)
            .collect();
        self.tasks.push(MsgTaskDialog {
            task_id: 0,
            avatar: 0,
            option_id: u8::MAX,
            action: DialogActionKind::Text.into(),
            msgs,
        });
        MultiTaskDialogBuilder {
            tasks: self.tasks,
            _state: std::marker::PhantomData,
        }
    }
}

impl MultiTaskDialogBuilder<AddingOptionOrEdit> {
    fn mk_option<T: AsRef<str>>(mut self, option_id: u8, text: T, action: DialogActionKind) -> Self {
        let s = text.as_ref();
        // Truncate the text to the maximum allowed length.
        let option_text = if s.len() <= constants::MAX_TXT_LEN {
            s
        } else {
            // Take the first MAX_TXT_LEN characters.
            match s.char_indices().nth(constants::MAX_TXT_LEN) {
                None => s,
                Some((idx, _)) => &s[..idx],
            }
        };
        self.tasks.push(MsgTaskDialog {
            task_id: 0,
            avatar: 0,
            option_id,
            action: action.into(),
            msgs: vec![option_text].into(),
        });
        self
    }

    /// Adds an option task to the list of tasks.
    ///
    /// # Arguments
    ///
    /// * `option_id` - A u8 representing the option id to be added to the task.
    /// * `text` - A string slice representing the text to be added to the task.
    pub fn with_option<T: AsRef<str>>(self, option_id: u8, text: T) -> Self {
        self.mk_option(option_id, text, DialogActionKind::Link)
    }

    /// Adds an edit task to the list of tasks.
    ///
    /// # Arguments
    ///
    /// * `option_id` - A u8 representing the option id to be added to the task.
    /// * `text` - A string slice representing the text to be added to the task.
    pub fn with_edit<T: AsRef<str>>(self, option_id: u8, text: T) -> Self {
        self.mk_option(option_id, text, DialogActionKind::Edit)
    }

    /// Transitions to the next state.
    pub fn and(self) -> MultiTaskDialogBuilder<AddingAvatar> {
        MultiTaskDialogBuilder {
            tasks: self.tasks,
            _state: std::marker::PhantomData,
        }
    }
}

impl MultiTaskDialogBuilder<AddingAvatar> {
    /// Adds an avatar task to the list of tasks.
    ///
    /// # Arguments
    ///
    /// * `avatar` - A u16 representing the avatar to be added to the task.
    pub fn with_avatar(mut self, avatar: u16) -> MultiTaskDialogBuilder<Ready> {
        self.tasks.push(MsgTaskDialog {
            task_id: 0,
            avatar,
            option_id: u8::MAX,
            action: DialogActionKind::Avatar.into(),
            msgs: Default::default(),
        });
        MultiTaskDialogBuilder {
            tasks: self.tasks,
            _state: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl PacketProcess for MsgTaskDialog {
    type ActorState = crate::ActorState;
    type Error = crate::Error;
    type State = crate::State;

    async fn process(&self, _state: &Self::State, _actor: &Actor<Self::ActorState>) -> Result<(), Self::Error> {
        tracing::debug!(msg = ?self, "MsgTaskDialog received");
        Ok(())
    }
}
