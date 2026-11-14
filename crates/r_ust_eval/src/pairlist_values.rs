use crate::{EvalError, ListElement, Sexp};

pub(super) fn restore(value: Sexp) -> Result<Sexp, EvalError> {
    match value {
        Sexp::List(values) => Ok(if values.is_empty() {
            Sexp::Nil
        } else {
            Sexp::pairlist(values)
        }),
        _ => Err(EvalError::Type(
            "pairlist replacement requires list elements".into(),
        )),
    }
}

pub(super) fn assign_name(
    values: &[ListElement],
    name: &str,
    replacement: &Sexp,
) -> Result<Sexp, EvalError> {
    let mut values = values.to_vec();
    if let Some(index) = values
        .iter()
        .position(|slot| slot.name.as_deref() == Some(name))
    {
        if matches!(replacement, Sexp::Nil) {
            values.remove(index);
        } else {
            values[index].value = replacement.clone();
        }
    } else if !matches!(replacement, Sexp::Nil) {
        values.push(ListElement {
            name: Some(name.into()),
            value: replacement.clone(),
        });
    }
    restore(Sexp::List(values))
}

pub(super) fn assign_names(values: &[ListElement], replacement: &Sexp) -> Result<Sexp, EvalError> {
    let names = if matches!(replacement, Sexp::Nil) {
        Vec::new()
    } else {
        crate::string_values(replacement)?
    };
    if names.len() > values.len() {
        return Err(EvalError::Domain(format!(
            "'names' attribute [{}] must be the same length as the vector [{}]",
            names.len(),
            values.len()
        )));
    }
    let values = values
        .iter()
        .enumerate()
        .map(|(index, slot)| ListElement {
            name: if matches!(replacement, Sexp::Nil) {
                None
            } else {
                names
                    .get(index)
                    .cloned()
                    .flatten()
                    .filter(|name| !name.is_empty())
            },
            value: slot.value.clone(),
        })
        .collect();
    restore(Sexp::List(values))
}
