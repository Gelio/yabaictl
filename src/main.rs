use clap::{Parser, Subcommand};
use yabaictl::{
    cli::{
        focus_space::focus_space_by_index, focus_window_in_direction::focus_window_in_direction,
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

#[derive(Subcommand)]
enum Command {
    FocusSpace { index: u32 },
    FocusWindow { direction: Direction },
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Command::FocusSpace { index } => focus_space_by_index(SpaceIndex(index)),
        Command::FocusWindow { direction } => focus_window_in_direction(direction),
    }
}
