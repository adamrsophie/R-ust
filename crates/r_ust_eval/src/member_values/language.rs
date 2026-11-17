pub(super) mod index;
pub(super) mod selection;

use crate::{CallArgument, EvalError, ListElement, Sexp};
use r_ust_sexp::canonical_actual_call;

pub(super) fn extract(value: &Sexp, subscript: &Sexp) -> Result<Sexp, EvalError> {
    let elements = elements(value)?;
    let position = index::read(&elements, subscript)?;
    match position {
        None => Ok(Sexp::Nil),
        Some(position) => elements
            .get(position)
            .map(|slot| slot.value.clone())
            .ok_or_else(|| EvalError::Domain("subscript out of bounds".into())),
    }
}

pub(super) fn assign(value: &Sexp, subscript: &Sexp, rhs: &Sexp) -> Result<Sexp, EvalError> {
    let mut elements = elements(value)?;
    let deleting = matches!(rhs, Sexp::Nil);
    let selection = selection::select(&elements, subscript, !deleting)?;
    if deleting {
        let head_removed = selection.positions.contains(&Some(0));
        let retained = elements
            .into_iter()
            .enumerate()
            .filter(|(index, _)| !selection.positions.contains(&Some(*index)))
            .map(|(_, slot)| slot)
            .collect::<Vec<_>>();
        return if head_removed {
            Ok(if retained.is_empty() {
                Sexp::Nil
            } else {
                Sexp::pairlist(retained)
            })
        } else {
            restore(retained)
        };
    }
    if selection.positions.len() > 1 {
        return Err(EvalError::Domain("invalid subscript in list assign".into()));
    }
    if let Some(Some(position)) = selection.positions.first() {
        if *position >= elements.len() {
            let length = position
                .checked_add(1)
                .ok_or_else(|| EvalError::Domain("result would be too long a vector".into()))?;
            elements
                .try_reserve(length - elements.len())
                .map_err(|_| EvalError::Domain("cannot allocate language elements".into()))?;
            elements.resize_with(length, || ListElement {
                name: None,
                value: Sexp::Nil,
            });
            elements[*position].name = selection.new_tag;
        }
        elements[*position].value = rhs.clone();
    }
    restore(elements)
}

pub(super) fn elements(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    let canonical = canonical_actual_call(value)
        .map_err(|_| EvalError::NotImplemented("language member canonicalization"))?;
    let Sexp::Call {
        function,
        arguments,
    } = canonical
    else {
        return Err(EvalError::Type("language member requires a call".into()));
    };
    let mut values = vec![ListElement {
        name: None,
        value: *function,
    }];
    values.extend(arguments.into_iter().map(|argument| ListElement {
        name: argument.name,
        value: argument.value,
    }));
    Ok(values)
}

pub(super) fn restore(elements: Vec<ListElement>) -> Result<Sexp, EvalError> {
    let mut elements = elements.into_iter();
    let Some(head) = elements.next() else {
        return Ok(Sexp::Nil);
    };
    Ok(Sexp::Call {
        function: Box::new(head.value),
        arguments: elements
            .map(|slot| CallArgument {
                name: slot.name,
                value: slot.value,
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests;
