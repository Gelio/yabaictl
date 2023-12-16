use std::iter::zip;

use super::Labelable;

#[derive(Debug, PartialEq, Eq)]
pub struct PartitionedLabelables<T: Labelable> {
    incorrectly_labeled: Vec<(T, T::ParseIndexError)>,
    unused_indexes: Vec<u32>,
    items_to_label: Vec<T>,
}

impl<T: Labelable> PartitionedLabelables<T> {
    pub fn incorrectly_labeled(&self) -> &Vec<(T, T::ParseIndexError)> {
        &self.incorrectly_labeled
    }

    pub fn into_assigned_indices(self) -> AssignedIndices<T> {
        self.into()
    }
}

pub fn partition_labelables<T, const MAX_INDEX: usize>(
    labelables: impl Iterator<Item = T>,
) -> PartitionedLabelables<T>
where
    T: Labelable,
{
    let mut index_used = [false; MAX_INDEX];

    let mut items_with_invalid_labels: Vec<(T, T::ParseIndexError)> = Vec::new();

    let (labeled_items, items_to_label): (Vec<_>, Vec<_>) =
        labelables.partition(|labelable| labelable.label().is_some());

    for item in labeled_items.into_iter() {
        if let Some(label) = item.label() {
            match T::parse_index(label) {
                Ok(index) => {
                    index_used[index as usize] = true;
                }
                Err(error) => {
                    items_with_invalid_labels.push((item, error));
                }
            }
        }
    }

    let unused_indexes: Vec<_> = index_used
        .into_iter()
        .enumerate()
        // NOTE: skips the 0th index
        .skip(1)
        .filter_map(|(index, used)| if used { None } else { Some(index as u32) })
        .collect();

    PartitionedLabelables {
        incorrectly_labeled: items_with_invalid_labels,
        unused_indexes,
        items_to_label,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AssignedIndices<T> {
    pub assigned_indices: Vec<(T, u32)>,
    /// Items that could not be labeled because there are not enough labels.
    pub leftover_items: Vec<T>,
}

impl<T: Labelable> From<PartitionedLabelables<T>> for AssignedIndices<T> {
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
        type ParseIndexError = ParseIntError;

        fn label(&self) -> Option<&str> {
            self.1.as_deref()
        }

        fn parse_index(label: &str) -> Result<u32, Self::ParseIndexError> {
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

        let result = partition_labelables::<_, 7>(items.into_iter());
        assert_eq!(
            vec![LabelableItem(3, None), LabelableItem(4, None)],
            result.items_to_label,
        );
        assert_eq!(vec![3, 4, 5, 6], result.unused_indexes,);
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
