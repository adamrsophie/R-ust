use crate::{ListElement, Sexp};

pub(super) fn subset_list(values: &[ListElement], positions: &[Option<usize>]) -> Vec<ListElement> {
    positions
        .iter()
        .map(|position| {
            position
                .and_then(|position| values.get(position).cloned())
                .unwrap_or(ListElement {
                    name: None,
                    value: Sexp::Nil,
                })
        })
        .collect()
}
