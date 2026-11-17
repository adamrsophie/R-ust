use crate::{EvalError, ListElement, Sexp};

pub(super) fn frame(value: &Sexp) -> Result<Option<r_ust_sexp::Environment>, EvalError> {
    if value.sexptype() != r_ust_sexp::SexpType::Environment {
        return Ok(None);
    }
    let value = crate::ordinary_values::read(value)?;
    match value.as_ref() {
        Sexp::Environment(frame) => Ok(Some(frame.clone())),
        _ => Err(EvalError::Type("invalid environment transport".into())),
    }
}

pub(super) fn set(value: &Sexp, name: &str, replacement: &Sexp) -> Result<Sexp, EvalError> {
    let frame = frame(value)?.ok_or_else(|| EvalError::Type("expected environment".into()))?;
    if name.is_empty() {
        return Err(EvalError::Domain(
            "attempt to use zero-length variable name".into(),
        ));
    }
    if matches!(name, "names" | "dim" | "dimnames") {
        return Err(EvalError::Type(
            "invalid first argument, must be vector (list or atomic)".into(),
        ));
    }
    let mut replacement = replacement.clone();
    if name == "class" {
        let data = crate::coercion::storage::read(&replacement)?;
        if !matches!(data.as_ref(), Sexp::Nil) {
            let classes = data.character_values().ok_or_else(|| {
                EvalError::Type("attempt to set invalid 'class' attribute".into())
            })?;
            if classes.is_empty() {
                replacement = Sexp::Nil;
            }
        }
    }
    let mut attributes = frame.attribute_values()?;
    if let Some(index) = attributes
        .iter()
        .position(|attribute| attribute.name.as_deref() == Some(name))
    {
        if replacement.sexptype() == r_ust_sexp::SexpType::Nil {
            attributes.remove(index);
        } else {
            attributes[index].value = replacement;
        }
    } else if replacement.sexptype() != r_ust_sexp::SexpType::Nil {
        attributes.push(ListElement {
            name: Some(name.into()),
            value: replacement,
        });
    }
    frame.set_attributes(&attributes)?;
    Ok(value.clone())
}
