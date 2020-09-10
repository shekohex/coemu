use crate::{
    packets::{MsgTalk, TalkChannel},
    ActorState, Error,
};
use argh::FromArgs;
use tq_network::Actor;

pub async fn parse_and_execute(
    args: &[&str],
    actor: &Actor<ActorState>,
) -> Result<(), Error> {
    let me = actor.character().await?;
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
            me.teleport(info.map_id, (info.x, info.y)).await
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
    Teleport(TeleportCmd),
}

/// Disconnect From Server
#[derive(Debug, Clone, PartialEq, FromArgs)]
#[argh(subcommand, name = "dc")]
struct DcCmd {}

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
}
