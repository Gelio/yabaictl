use std::{iter::zip, usize};

use super::Labelable;

#[derive(Debug, PartialEq, Eq)]
pub struct PartitionedLabelables<T: Labelable> {
    incorrectly_labeled: Vec<(T, T::ParseIndexError)>,
    unused_indexes: Vec<T::Index>,
    items_to_label: Vec<T>,
}

impl<T: Labelable> PartitionedLabelables<T> {
    pub fn incorrectly_labeled(&self) -> &Vec<(T, T::ParseIndexError)> {
        &self.incorrectly_labeled
    }

    pub fn into_assigned_indices(self) -> AssignedIndices<T, T::Index> {
        self.into()
    }
}

pub fn partition_labelables<T>(labelables: impl Iterator<Item = T>) -> PartitionedLabelables<T>
where
    T: Labelable,
    <<T as Labelable>::Index as TryFrom<u32>>::Error: std::fmt::Debug,
{
    // NOTE: cannot use an array because cannot add 1 to a const generic without a nightly flag
    let mut index_used: Vec<bool> = vec![false; *T::INDEX_RANGE.end() as usize + 1];

    let mut items_with_invalid_labels: Vec<(T, T::ParseIndexError)> = Vec::new();

    let (labeled_items, items_to_label): (Vec<_>, Vec<_>) =
        labelables.partition(|labelable| labelable.label().is_some());

    for item in labeled_items.into_iter() {
        if let Some(label) = item.label() {
            match T::parse_index(label) {
                Ok(index) => {
                    index_used[index.into() as usize] = true;
                }
                Err(error) => {
                    items_with_invalid_labels.push((item, error));
                }
            }
        }
    }

    let unused_indexes: Vec<_> = T::INDEX_RANGE
        .filter_map(|index| {
            if index_used[index as usize] {
                None
            } else {
                Some(
                    index
                        .try_into()
                        .expect("Cannot convert an unused index to labelable index"),
                )
            }
        })
        .collect();

    PartitionedLabelables {
        incorrectly_labeled: items_with_invalid_labels,
        unused_indexes,
        items_to_label,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssignedIndices<T, Index> {
    pub assigned_indices: Vec<(T, Index)>,
    /// Items that could not be labeled because there are not enough labels.
    pub leftover_items: Vec<T>,
}

impl<T: Labelable> From<PartitionedLabelables<T>> for AssignedIndices<T, T::Index> {
    fn from(value: PartitionedLabelables<T>) -> Self {
        let mut items_to_label = value.items_to_label;

        Self {
            leftover_items: items_to_label
                .split_off(value.unused_indexes.len().min(items_to_label.len())),
            assigned_indices: zip(items_to_label, value.unused_indexes).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::ParseIntError;

    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct LabelableItem(u32, Option<String>);

    impl Labelable for LabelableItem {
        const INDEX_RANGE: std::ops::RangeInclusive<u32> = 1..=6;
        type Index = u32;
        type ParseIndexError = ParseIntError;

        fn label(&self) -> Option<&str> {
            self.1.as_deref()
        }

        fn parse_index(label: &str) -> Result<Self::Index, Self::ParseIndexError> {
            label.parse()
        }
    }

    #[test]
    fn partitions_items() {
        let items = vec![
            LabelableItem(1, Some("1".to_owned())),
            LabelableItem(2, Some("2".to_owned())),
            LabelableItem(3, None),
            LabelableItem(4, None),
            LabelableItem(5, Some("invalid one".to_owned())),
        ];

        let result = partition_labelables(items.into_iter());
        assert_eq!(
            vec![LabelableItem(3, None), LabelableItem(4, None)],
            result.items_to_label,
        );
        assert_eq!(vec![3, 4, 5, 6], result.unused_indexes);
        assert_eq!(1, result.incorrectly_labeled.len());
        assert_eq!(
            result.incorrectly_labeled[0].0,
            LabelableItem(5, Some("invalid one".to_owned())),
        );
    }

    #[test]
    fn assign_indices_for_all_items() {
        let result = PartitionedLabelables {
            incorrectly_labeled: Vec::new(),
            unused_indexes: vec![7, 8, 9],
            items_to_label: vec![LabelableItem(3, None), LabelableItem(4, None)],
        }
        .into_assigned_indices();

        assert_eq!(
            AssignedIndices {
                assigned_indices: vec![(LabelableItem(3, None), 7), (LabelableItem(4, None), 8)],
                leftover_items: Vec::new()
            },
            result
        );
    }

    #[test]
    fn assign_indices_when_more_items_than_labels() {
        let result = PartitionedLabelables {
            incorrectly_labeled: Vec::new(),
            unused_indexes: vec![7],
            items_to_label: vec![LabelableItem(3, None), LabelableItem(4, None)],
        }
        .into_assigned_indices();

        assert_eq!(
            AssignedIndices {
                assigned_indices: vec![(LabelableItem(3, None), 7)],
                leftover_items: vec!(LabelableItem(4, None))
            },
            result
        );
    }
}
