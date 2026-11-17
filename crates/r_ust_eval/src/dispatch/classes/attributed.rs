use crate::{EvalError, Sexp};

pub(super) fn classes(value: &Sexp) -> Result<Sexp, EvalError> {
    let attributes = crate::list_attributes::engine::stored(value)?;
    if let Some(class) = attributes
        .iter()
        .find(|attr| attr.name.as_deref() == Some("class"))
    {
        let class = crate::ordinary_values::read(&class.value)?;
        let values = class
            .character_values()
            .ok_or_else(|| EvalError::Type("invalid class attribute".into()))?;
        if !values.is_empty() {
            return Ok(class.into_owned());
        }
    }
    let payload = crate::coercion::storage::read(value)?;
    let fallback = super::classes(payload.as_ref())?;
    let dimensions = attributes
        .iter()
        .find(|attr| attr.name.as_deref() == Some("dim"))
        .map(|attr| crate::value_len(&attr.value))
        .transpose()?
        .unwrap_or(0);
    if dimensions == 0 {
        return Ok(fallback);
    }
    let suffix = fallback
        .character_values()
        .ok_or_else(|| EvalError::Type("invalid implicit class chain".into()))?;
    let mut classes = Vec::new();
    if dimensions == 2 {
        classes.push(Some("matrix".into()));
    }
    classes.push(Some("array".into()));
    classes.extend(suffix.iter().cloned());
    Ok(Sexp::String(classes))
}
