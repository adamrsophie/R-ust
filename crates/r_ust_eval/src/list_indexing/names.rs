use crate::{ListElement, Sexp};

pub(super) fn ordinary_names_subset(
    values: &[ListElement],
    names: &[Option<String>],
) -> Vec<ListElement> {
    names
        .iter()
        .map(|name| {
            name.as_deref()
                .filter(|name| !name.is_empty())
                .and_then(|name| {
                    values
                        .iter()
                        .find(|element| element.name.as_deref() == Some(name))
                })
                .cloned()
                .unwrap_or(ListElement {
                    name: None,
                    value: Sexp::Nil,
                })
        })
        .collect()
}
