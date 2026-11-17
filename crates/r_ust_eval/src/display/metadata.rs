use crate::{EvalError, ListElement, Sexp};

pub(super) fn get(attributes: &[ListElement], name: &str) -> Result<Option<Sexp>, EvalError> {
    attributes
        .iter()
        .find(|a| a.name.as_deref() == Some(name))
        .map(|a| super::projection::read(&a.value).map(|v| v.into_owned()))
        .transpose()
}

pub(super) fn strings(value: &Sexp) -> Result<Vec<Option<String>>, EvalError> {
    let value = super::projection::read(value)?;
    value
        .character_values()
        .map(<[_]>::to_vec)
        .ok_or_else(|| EvalError::Type("invalid character print metadata".into()))
}

pub(super) fn elements(
    values: &[Sexp],
    attributes: &[ListElement],
) -> Result<Vec<ListElement>, EvalError> {
    let names = super::names::read(attributes)?;
    Ok(values
        .iter()
        .enumerate()
        .map(|(index, value)| ListElement {
            name: names.as_ref().and_then(|v| v.get(index)).cloned().flatten(),
            value: value.clone(),
        })
        .collect())
}
