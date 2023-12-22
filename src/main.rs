use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use yabaictl::{
    cli::{
        focus_space::{
            focus_next_or_previous_space, focus_space_by_index, focus_space_by_label,
            NextOrPrevious,
        },
        focus_window_in_direction::focus_window_in_direction,
        label_spaces::label_spaces,
        move_space_in_direction::move_space_in_direction,
        move_window_to_space::move_window_to_space,
        reorder::reorder_spaces_by_stable_indexes,
        set_space_label::{set_space_label, SetSpaceLabelArgs},
    },
    label::space::StableSpaceIndex,
    position::Direction,
    simple_bar,
    yabai::transport::{Space, SpaceIndex},
};

#[derive(Parser)]
#[command(author, version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
struct SpaceSpecifier {
    next_or_previous: Option<NextOrPrevious>,

    #[arg(long = "index")]
    index: Option<u32>,

    #[arg(long = "label")]
    label_prefix: Option<String>,

    #[arg(long = "stable-index")]
    stable_index: Option<StableSpaceIndex>,
}

#[derive(Subcommand)]
enum Command {
    FocusSpace {
        #[command(flatten)]
        space_specifier: SpaceSpecifier,

        #[arg(short, long, default_value_t = false)]
        create_if_not_found: bool,
    },
    FocusWindow {
        direction: Direction,
    },
    MoveSpace {
        direction: Direction,
    },
    LabelSpaces,
    ReorderByStableIndexes,
    SetLabel(SetSpaceLabelArgs),
    MoveWindow {
        stable_space_index: StableSpaceIndex,

        #[arg(short, long, default_value_t = false)]
        create_if_not_found: bool,
    },
    // TODO: warp (move) window in a given direction
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Command::FocusSpace {
            space_specifier,
            create_if_not_found: create_space_if_not_found,
        } => {
            if let Some(index) = space_specifier.index {
                focus_space_by_index(SpaceIndex(index))
            } else if let Some(next_or_previous) = space_specifier.next_or_previous {
                focus_next_or_previous_space(next_or_previous)
            } else if let Some(label_prefix) = space_specifier.label_prefix {
                focus_space_by_label(&label_prefix, create_space_if_not_found)
            } else if let Some(stable_index) = space_specifier.stable_index {
                let label_prefix = Space::label(stable_index, None);
                focus_space_by_label(&label_prefix, create_space_if_not_found)
            } else {
                unreachable!("Some space specifier is required");
            }
        }
        Command::FocusWindow { direction } => focus_window_in_direction(direction),
        Command::MoveSpace { direction } => {
            move_space_in_direction(direction).and_then(|_| reorder_spaces_by_stable_indexes())
        }
        Command::LabelSpaces => label_spaces().and_then(|_| reorder_spaces_by_stable_indexes()),
        Command::ReorderByStableIndexes => reorder_spaces_by_stable_indexes(),
        Command::SetLabel(args) => {
            set_space_label(args).and_then(|_| reorder_spaces_by_stable_indexes())
        }
        Command::MoveWindow {
            stable_space_index,
            create_if_not_found: create_space_if_not_found,
        } => move_window_to_space(stable_space_index, create_space_if_not_found),
    }
    .and_then(|_| simple_bar::update().context("Cannot update simple-bar"))
}
