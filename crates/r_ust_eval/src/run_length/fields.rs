use super::{select, storage};
use crate::{EvalError, ListElement, Sexp};

pub(super) fn member(value: &Sexp, name: &str) -> Result<Sexp, EvalError> {
    let plain = storage::payload(value)?;
    let plain = if let Sexp::Expression { elements, .. } = plain {
        Sexp::Vector(elements.into_iter().map(|e| e.value).collect())
    } else {
        plain
    };
    match plain {
        Sexp::Nil => Ok(Sexp::Nil),
        Sexp::Environment(frame) => Ok(frame.get_local(name)?.unwrap_or(Sexp::Nil)),
        Sexp::PairList(values) => Ok(lookup(values.as_slice(), name)),
        Sexp::Vector(values) => {
            let names = select::names(value)?.unwrap_or_default();
            let values = values
                .into_iter()
                .enumerate()
                .map(|(i, value)| ListElement {
                    name: names.get(i).cloned().flatten(),
                    value,
                })
                .collect::<Vec<_>>();
            Ok(lookup(&values, name))
        }
        Sexp::Call {
            function,
            arguments,
        } => {
            let mut values = vec![ListElement {
                name: None,
                value: *function,
            }];
            values.extend(arguments.into_iter().map(|a| ListElement {
                name: a.name,
                value: a.value,
            }));
            Ok(lookup(&values, name))
        }
        Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_) => Err(
            EvalError::Type("$ operator is invalid for atomic vectors".into()),
        ),
        other => Err(EvalError::Type(format!(
            "object of type '{}' is not subsettable",
            crate::metadata::typeof_value(&other)
        ))),
    }
}

pub(super) fn length(value: &Sexp) -> Result<usize, EvalError> {
    let plain = storage::payload(value)?;
    if let Sexp::Environment(frame) = plain {
        return Ok(frame.local_len()?);
    }
    crate::value_len(&plain)
}

fn lookup(values: &[ListElement], name: &str) -> Sexp {
    if let Some(value) = values.iter().find(|e| e.name.as_deref() == Some(name)) {
        return value.value.clone();
    }
    let mut matches = values
        .iter()
        .filter(|e| e.name.as_deref().is_some_and(|n| n.starts_with(name)));
    match (matches.next(), matches.next()) {
        (Some(value), None) => value.value.clone(),
        _ => Sexp::Nil,
    }
}
