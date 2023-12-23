use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use yabaictl::{
    cli::{
        destroy_spaces,
        focus_space::{focus_next_or_previous_space, focus_space_by_label, NextOrPrevious},
        focus_window_in_direction::focus_window_in_direction,
        label_spaces::label_spaces,
        move_space_in_direction::move_space_in_direction,
        move_window_in_direction::move_window_in_direction,
        move_window_to_space::move_window_to_space,
        reorder::reorder_spaces_by_stable_indexes,
        set_space_label::{set_space_label, SetSpaceLabelArgs},
    },
    label::space::StableSpaceIndex,
    position::Direction,
    simple_bar,
    yabai::transport::Space,
};

#[derive(Parser)]
#[command(author, about, version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Args, Clone)]
#[group(required = true, multiple = false)]
struct FocusSpaceSpecifier {
    /// Next or previous space based on the active one.
    /// Wraps within the display.
    next_or_previous: Option<NextOrPrevious>,

    /// Prefix for the label. It could be the full label itself.
    #[arg(long)]
    label_prefix: Option<String>,

    #[arg(long)]
    stable_index: Option<StableSpaceIndex>,
}

#[derive(Args)]
struct TargetSpaceUsingStableIndexOptions {
    /// If the target space does not exist, it will be created before focusing it.
    /// The space will belong to the currently active display.
    #[arg(long, default_value_t = false)]
    create_if_not_found: bool,
}

#[derive(Subcommand)]
enum MoveWindowSpaceSpecifier {
    /// Move the window to a desired space, specified by a stable index.
    ToSpace {
        stable_space_index: StableSpaceIndex,

        #[command(flatten)]
        target_space_options: TargetSpaceUsingStableIndexOptions,
    },
    /// Move the window in a given direction.
    /// Supports moving the window across displays.
    InDirection { direction: Direction },
}

#[derive(Subcommand)]
enum Command {
    /// Focuses a space.
    FocusSpace {
        #[command(flatten)]
        space_specifier: FocusSpaceSpecifier,

        #[command(flatten)]
        target_space_options: TargetSpaceUsingStableIndexOptions,

        /// Spaces with 0 windows that are in the background (not visible) are going
        /// to be removed.
        ///
        /// This helps manage the spaces, since unused spaces won't take up slots on displays.
        ///
        /// Useful with the `create_if_not_found` option.
        #[arg(long, default_value_t = false)]
        destroy_empty_background_spaces: bool,
    },
    /// Focuses a window in a given direction based on the active window.
    /// Works across displays.
    FocusWindow { direction: Direction },
    /// Move the active space in a given direction across displays.
    MoveSpace { direction: Direction },
    /// Assigns stable indexes to spaces using labels.
    LabelSpaces,
    /// Reorders spaces using their stable indexes, parsed from their labels.
    ReorderByStableIndexes,
    /// Assigns a label to a space.
    SetLabel(SetSpaceLabelArgs),
    /// Move the currently active window to another space or in a given direction.
    MoveWindow {
        #[command(subcommand)]
        space_specifier: MoveWindowSpaceSpecifier,
    },
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();

    match cli.command {
        Command::FocusSpace {
            space_specifier,
            target_space_options,
            destroy_empty_background_spaces,
        } => {
            if let Some(next_or_previous) = space_specifier.next_or_previous {
                focus_next_or_previous_space(next_or_previous)?;
            } else if let Some(label_prefix) = space_specifier.label_prefix {
                focus_space_by_label(&label_prefix, target_space_options.create_if_not_found)?;
            } else if let Some(stable_index) = space_specifier.stable_index {
                let label_prefix = Space::label(stable_index, None);
                focus_space_by_label(&label_prefix, target_space_options.create_if_not_found)?;
            } else {
                unreachable!("Some space specifier is required");
            }

            if destroy_empty_background_spaces {
                destroy_spaces::destroy_empty_background_spaces()?;
            }

            Ok(())
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
        Command::MoveWindow { space_specifier } => match space_specifier {
            MoveWindowSpaceSpecifier::ToSpace {
                stable_space_index,
                target_space_options,
            } => move_window_to_space(stable_space_index, target_space_options.create_if_not_found)
                .and_then(|_| reorder_spaces_by_stable_indexes()),
            MoveWindowSpaceSpecifier::InDirection { direction } => {
                move_window_in_direction(direction)
            }
        },
    }
    .and_then(|_| simple_bar::update().context("Cannot update simple-bar"))
}

#[cfg(test)]
#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}
