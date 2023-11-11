use clap::{Parser, Subcommand};
use yabaictl::{cli::focus_space::focus_space_by_index, yabai::transport::SpaceIndex};

#[derive(Parser)]
#[command(author, version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    FocusSpace { index: u32 },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::FocusSpace { index } => focus_space_by_index(SpaceIndex(index)),
    }
}
