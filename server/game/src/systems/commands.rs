use crate::packets::{MsgTalk, TalkChannel};
use crate::world::Maps;
use crate::{ActorState, Error};
use argh::FromArgs;
use tq_network::Actor;

pub async fn parse_and_execute(
    state: &crate::State,
    actor: &Actor<ActorState>,
    args: &[&str],
) -> Result<(), Error> {
    let me = actor.character();
    let c = match Command::from_args(&["commands"], args) {
        Ok(cmd) => cmd,
        Err(e) => {
            let lines = e
                .output
                .lines()
                .map(|e| e.to_owned())
                .skip_while(|e| e.is_empty());
            for line in lines {
                actor
                    .send(MsgTalk::from_system(
                        me.id(),
                        TalkChannel::System,
                        line,
                    ))
                    .await?;
            }
            return Ok(());
        },
    };
    match c.commands {
        SubCommands::Dc(_) => {
            actor.shutdown().await?;
            Ok(())
        },
        SubCommands::Teleport(info) => {
            let old_map = state.try_map(me.map_id())?;
            let map = state.try_map(info.map_id)?;
            me.teleport(state, info.map_id, (info.x, info.y)).await?;
            map.insert_character(me.clone()).await?;
            old_map.remove_character(&me)?;
            if info.all {
                let others = state.characters();
                for other in others {
                    if other.id() != me.id() {
                        other
                            .teleport(state, info.map_id, (info.x, info.y))
                            .await?;
                        map.insert_character(other.clone()).await?;
                        old_map.remove_character(&other)?;
                    }
                }
            }
            Ok(())
        },
        SubCommands::Which(which) => {
            if which.map {
                let map_id = me.map_id();
                actor
                    .send(MsgTalk::from_system(
                        me.id(),
                        TalkChannel::System,
                        format!(
                            "Current Map: {:?} = {}",
                            Maps::from(map_id),
                            map_id
                        ),
                    ))
                    .await?;
            }
            Ok(())
        },
        SubCommands::JumpBack(_) => {
            me.kick_back().await?;
            Ok(())
        },
    }
}

/// In Game Commands
#[derive(Debug, Clone, PartialEq, FromArgs)]
struct Command {
    #[argh(subcommand)]
    commands: SubCommands,
}

#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand)]
enum SubCommands {
    Dc(DcCmd),
    Which(WhichCmd),
    Teleport(TeleportCmd),
    JumpBack(JumpBackCmd),
}

/// Disconnect From Server
#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name = "dc")]
struct DcCmd {}

/// Jump Back to prev location
#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name = "jump-back")]
struct JumpBackCmd {}

/// Ask about things in your environment
#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name = "which")]
struct WhichCmd {
    /// get your current map (ID, Name)
    #[argh(switch)]
    map: bool,
}

/// Teleport to other map at specific location
#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name = "tele")]
struct TeleportCmd {
    #[argh(positional)]
    map_id: u32,
    #[argh(positional)]
    x: u16,
    #[argh(positional)]
    y: u16,
    /// teleport all characters with you
    #[argh(option, default = "false")]
    all: bool,
}
