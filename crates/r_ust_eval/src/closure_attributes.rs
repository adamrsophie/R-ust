use crate::{EvalError, ListElement, Sexp};

pub(super) fn attributes(value: &Sexp) -> Result<&[ListElement], EvalError> {
    match value {
        Sexp::Closure { attributes, .. } => Ok(attributes),
        _ => Err(EvalError::Type("expected closure".into())),
    }
}

pub(super) fn explicit_class(value: &Sexp) -> Option<&Sexp> {
    let Sexp::Closure { attributes, .. } = value else {
        return None;
    };
    attributes
        .iter()
        .find(|attribute| attribute.name.as_deref() == Some("class"))
        .map(|attribute| &attribute.value)
}

pub(super) fn replace(value: &Sexp, replacements: &[ListElement]) -> Result<Sexp, EvalError> {
    let mut result = value.clone();
    let Sexp::Closure { attributes, .. } = &mut result else {
        return Err(EvalError::Type("expected closure".into()));
    };
    attributes.clear();
    for attribute in replacements {
        result = set(
            &result,
            attribute.name.as_deref().unwrap_or(""),
            &attribute.value,
        )?;
    }
    Ok(result)
}

pub(super) fn set(value: &Sexp, name: &str, replacement: &Sexp) -> Result<Sexp, EvalError> {
    if name.is_empty() {
        return Err(EvalError::Domain(
            "attempt to use zero-length variable name".into(),
        ));
    }
    if matches!(
        name,
        "names" | "dim" | "dimnames" | "levels" | "tsp" | "row.names" | "comment"
    ) {
        return Err(EvalError::NotImplemented("special closure attributes"));
    }
    let replacement = if name == "class" {
        normalize_class(replacement)?
    } else {
        replacement.clone()
    };
    let mut result = value.clone();
    let Sexp::Closure { attributes, .. } = &mut result else {
        return Err(EvalError::Type("expected closure".into()));
    };
    if let Some(index) = attributes
        .iter()
        .position(|attribute| attribute.name.as_deref() == Some(name))
    {
        if matches!(replacement, Sexp::Nil) {
            attributes.remove(index);
        } else {
            attributes[index].value = replacement;
        }
    } else if !matches!(replacement, Sexp::Nil) {
        attributes.push(ListElement {
            name: Some(name.into()),
            value: replacement,
        });
    }
    Ok(result)
}

fn normalize_class(value: &Sexp) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::Nil) {
        return Ok(Sexp::Nil);
    }
    let values = value
        .character_values()
        .ok_or_else(|| EvalError::Type("attempt to set invalid 'class' attribute".into()))?;
    if values.iter().any(|name| name.as_deref() == Some("factor")) {
        return Err(EvalError::Type(
            "adding class \"factor\" to an invalid object".into(),
        ));
    }
    Ok(if values.is_empty() {
        Sexp::Nil
    } else {
        value.clone()
    })
}
