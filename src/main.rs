use clap::{Args, Parser, Subcommand};
use yabaictl::{
    cli::{
        focus_space::{
            focus_next_or_previous_space, focus_space_by_index, focus_space_by_label,
            NextOrPrevious,
        },
        focus_window_in_direction::focus_window_in_direction,
        move_space_in_direction::move_space_in_direction,
    },
    position::Direction,
    yabai::transport::SpaceIndex,
};

#[derive(Parser)]
#[command(author, version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct SpaceSpecifier {
    next_or_previous: Option<NextOrPrevious>,

    #[arg(long = "index")]
    index: Option<u32>,

    #[arg(long = "label")]
    label_prefix: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    FocusSpace(SpaceSpecifier),
    FocusWindow { direction: Direction },
    MoveSpace { direction: Direction },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Command::FocusSpace(space_specifier) => {
            if let Some(index) = space_specifier.index {
                focus_space_by_index(SpaceIndex(index))
            } else if let Some(next_or_previous) = space_specifier.next_or_previous {
                focus_next_or_previous_space(next_or_previous)
            } else if let Some(label_prefix) = space_specifier.label_prefix {
                focus_space_by_label(&label_prefix)
            } else {
                unreachable!("Some space specifier is required");
            }
        }
        Command::FocusWindow { direction } => focus_window_in_direction(direction),
        Command::MoveSpace { direction } => move_space_in_direction(direction),
    }
}
