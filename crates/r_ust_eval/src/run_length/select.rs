use super::storage;
use crate::{EvalError, ListElement, Sexp};

pub(super) fn positions(
    value: &Sexp,
    positions: &[Option<usize>],
    keep_names: bool,
) -> Result<Sexp, EvalError> {
    let plain = storage::payload(value)?;
    let plain = if let Sexp::PairList(values) = plain {
        Sexp::List(values.as_slice().to_vec())
    } else {
        plain
    };
    let result = crate::subset_linear_positions(&plain, positions)?;
    let mut attributes = Vec::new();
    if keep_names {
        if let Some(names) = names(value)? {
            attributes.push(ListElement {
                name: Some("names".into()),
                value: Sexp::String(
                    positions
                        .iter()
                        .map(|i| i.and_then(|i| names.get(i)).cloned().flatten())
                        .collect(),
                ),
            });
        }
    } else {
        let original = storage::attributes(value)?;
        let class = original
            .iter()
            .find(|a| a.name.as_deref() == Some("class"))
            .map(|a| storage::payload(&a.value))
            .transpose()?;
        let classes = class.as_ref().and_then(Sexp::character_values);
        if classes.is_some_and(|v| v.iter().any(|v| v.as_deref() == Some("factor"))) {
            let ordered =
                classes.is_some_and(|v| v.iter().any(|v| v.as_deref() == Some("ordered")));
            let class = if ordered {
                vec![Some("ordered".into()), Some("factor".into())]
            } else {
                vec![Some("factor".into())]
            };
            attributes.push(ListElement {
                name: Some("class".into()),
                value: Sexp::String(class),
            });
            if let Some(levels) = original
                .iter()
                .find(|a| a.name.as_deref() == Some("levels"))
            {
                attributes.push(levels.clone());
            }
        }
    }
    decorate(result, attributes)
}

pub(super) fn decorate(value: Sexp, attributes: Vec<ListElement>) -> Result<Sexp, EvalError> {
    if attributes.is_empty() {
        return Ok(value);
    }
    Ok(match value.attribute_description()? {
        Some(Sexp::AttributedAtomic { values, .. }) => {
            Sexp::AttributedAtomic { values, attributes }
        }
        Some(Sexp::AttributedAggregate { values, .. }) => {
            Sexp::AttributedAggregate { values, attributes }
        }
        _ => return Err(EvalError::Type("run-length result must be a vector".into())),
    })
}

pub(super) fn names(value: &Sexp) -> Result<Option<Vec<Option<String>>>, EvalError> {
    let attributes = storage::attributes(value)?;
    let Some(names) = attributes
        .iter()
        .find(|a| a.name.as_deref() == Some("names"))
    else {
        return Ok(None);
    };
    let names = storage::payload(&names.value)?;
    let names = names
        .character_values()
        .ok_or_else(|| EvalError::Type("invalid 'names' attribute".into()))?;
    Ok(Some(names.to_vec()))
}
