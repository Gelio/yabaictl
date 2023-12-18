use anyhow::Context;
use itertools::Itertools;

use crate::{
    label::{space::StableSpaceIndex, Labelable},
    yabai::{self, cli::execute_yabai_cmd, transport::Space},
};

pub fn reorder_spaces_by_stable_indexes() -> anyhow::Result<()> {
    let spaces = execute_yabai_cmd(&yabai::command::QuerySpaces {
        only_current_display: false,
    })
    .context("Cannot query yabai spaces")?
    .context("Cannot parse yabai spaces")?;

    let (spaces_with_stable_indices, space_index_parsing_errors): (Vec<_>, Vec<_>) = spaces
        .into_iter()
        .filter_map(|space| {
            let stable_index = Space::parse_index(space.label.as_deref()?);

            Some(stable_index.map(|stable_index| (space, stable_index)))
        })
        .partition(Result::is_ok);
    let spaces_with_stable_indices: Vec<_> = spaces_with_stable_indices
        .into_iter()
        .map(Result::unwrap)
        .collect();
    let space_index_parsing_errors: Vec<_> = space_index_parsing_errors
        .into_iter()
        .map(Result::unwrap_err)
        .collect();
    anyhow::ensure!(
        space_index_parsing_errors.is_empty(),
        "Spaces stable index cannot be parsed: {space_index_parsing_errors:?}",
    );

    let spaces_by_display = spaces_with_stable_indices
        .into_iter()
        .group_by(|(space, _)| space.display_index);

    for (_, spaces) in spaces_by_display.into_iter() {
        let stable_indexes: Vec<_> = spaces.map(|(_, stable_index)| stable_index).collect();
        let move_list = generate_move_list(&stable_indexes);

        for m in move_list {
            let move_command: yabai::command::MoveSpace = m.into();
            execute_yabai_cmd(&move_command).with_context(|| {
                format!(
                    "Cannot move space {:?} before space {:?}",
                    move_command.source_index, move_command.target_index
                )
            })?;
        }
    }

    Ok(())
}

/// An instruction to move `source` BEFORE `target`.
/// This is how yabai's space `--move` command works.
struct Move<Item> {
    source: Item,
    target: Item,
}

impl From<Move<StableSpaceIndex>> for yabai::command::MoveSpace {
    fn from(value: Move<StableSpaceIndex>) -> Self {
        Self {
            source_index: value.source,
            target_index: value.target,
        }
    }
}

fn generate_move_list<Item: Eq + Ord + Clone>(items: &[Item]) -> Vec<Move<Item>> {
    if items.is_empty() {
        return Vec::new();
    }

    let mut greatest_encountered = &items[0];
    let mut moves: Vec<Move<Item>> = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let greatest_item_so_far = item >= greatest_encountered;
        greatest_encountered = greatest_encountered.max(item);
        if greatest_item_so_far {
            continue;
        }

        let target = items[0..index]
            .iter()
            .filter(|other| *other > item)
            .min()
            .expect(
                "The currently processed item is not the greatest one among the preceding items",
            );

        moves.push(Move {
            source: item.clone(),
            target: target.clone(),
        })
    }

    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simulate_moves<Item: Eq>(items: &mut Vec<Item>, moves: &[Move<Item>]) {
        for m in moves {
            let source_index = items
                .iter()
                .position(|item| item == &m.source)
                .expect("Move source must exist in items");

            let item = items.remove(source_index);

            let target_index = items
                .iter()
                .position(|item| item == &m.target)
                .expect("Move target must exist in items");

            items.insert(target_index, item);
        }
    }

    struct GenerateMoveListTestCase {
        items: Vec<u32>,
        expected: Vec<u32>,
        expected_moves_count: usize,
    }

    impl GenerateMoveListTestCase {
        fn run(mut self) {
            let moves = generate_move_list(&self.items);
            simulate_moves(&mut self.items, &moves);

            assert_eq!(self.expected, self.items);
            assert_eq!(self.expected_moves_count, moves.len());
        }
    }

    #[test]
    fn moves_op_op() {
        GenerateMoveListTestCase {
            items: vec![1, 2, 3],
            expected: vec![1, 2, 3],

            expected_moves_count: 0,
        }
        .run();
    }

    #[test]
    fn single_move() {
        GenerateMoveListTestCase {
            items: vec![2, 1, 3],
            expected: vec![1, 2, 3],

            expected_moves_count: 1,
        }
        .run();
    }

    #[test]
    fn five_spaces_many_moves() {
        GenerateMoveListTestCase {
            items: vec![5, 2, 1, 4, 3],
            expected: vec![1, 2, 3, 4, 5],

            expected_moves_count: 4,
        }
        .run();
    }

    #[test]
    fn five_spaces_almost_ordered() {
        GenerateMoveListTestCase {
            items: vec![2, 1, 4, 5, 3],
            expected: vec![1, 2, 3, 4, 5],

            expected_moves_count: 2,
        }
        .run();
    }
}
