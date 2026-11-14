use crate::{EvalError, ListElement, Sexp};

mod validation;
pub(super) use validation::validate;

fn validate_name(name: &str) -> Result<(), EvalError> {
    if name.is_empty() {
        return Err(EvalError::Domain(
            "attempt to use zero-length variable name".into(),
        ));
    }
    if matches!(
        name,
        "names" | "class" | "dim" | "dimnames" | "levels" | "tsp" | "row.names" | "comment"
    ) {
        return Err(EvalError::NotImplemented("special character attributes"));
    }
    Ok(())
}

pub(super) fn attributes(value: &Sexp) -> Result<&[ListElement], EvalError> {
    validate(value)?;
    match value {
        Sexp::AttributedCharacter { attributes, .. } => Ok(attributes),
        _ => Ok(&[]),
    }
}

pub(super) fn with_values(original: &Sexp, values: Vec<Option<String>>) -> Sexp {
    match original {
        Sexp::AttributedCharacter { attributes, .. } if !attributes.is_empty() => {
            Sexp::AttributedCharacter {
                values,
                attributes: attributes.clone(),
            }
        }
        _ => Sexp::String(values),
    }
}

pub(super) fn replace(value: &Sexp, replacements: &[ListElement]) -> Result<Sexp, EvalError> {
    validate(value)?;
    let Some(values) = value.character_values() else {
        return Err(EvalError::Type("expected character data".into()));
    };
    let mut result = Sexp::String(values.to_vec());
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
    validate_name(name)?;
    let mut attributes = attributes(value)?.to_vec();
    let Some(values) = value.character_values() else {
        return Err(EvalError::NotImplemented(
            "ordinary attributes on this value",
        ));
    };
    if let Some(index) = attributes
        .iter()
        .position(|attribute| attribute.name.as_deref() == Some(name))
    {
        if matches!(replacement, Sexp::Nil) {
            attributes.remove(index);
        } else {
            attributes[index].value = replacement.clone();
        }
    } else if !matches!(replacement, Sexp::Nil) {
        attributes.push(ListElement {
            name: Some(name.into()),
            value: replacement.clone(),
        });
    }
    let result = if attributes.is_empty() {
        Sexp::String(values.to_vec())
    } else {
        Sexp::AttributedCharacter {
            values: values.to_vec(),
            attributes,
        }
    };
    validate(&result)?;
    Ok(result)
}

pub(super) fn get(value: &Sexp, name: &str, exact: bool) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::Char(_)) && (name == "names" || (!exact && "names".starts_with(name)))
    {
        return Err(EvalError::Domain(
            "cannot have attributes on a CHARSXP".into(),
        ));
    }
    let attrs = crate::list_attributes::attributes(value)?;
    let Sexp::List(attrs) = attrs else {
        return Ok(Sexp::Nil);
    };
    if let Some(attribute) = attrs
        .iter()
        .find(|attribute| attribute.name.as_deref() == Some(name))
    {
        return Ok(attribute.value.clone());
    }
    if !exact {
        let mut matches = attrs.iter().filter(|attribute| {
            attribute
                .name
                .as_deref()
                .is_some_and(|key| key.starts_with(name))
        });
        if let (Some(attribute), None) = (matches.next(), matches.next()) {
            return Ok(attribute.value.clone());
        }
    }
    Ok(Sexp::Nil)
}

pub(super) fn validate_class(class: &Sexp) -> Result<(), EvalError> {
    validation::check_class(class)?;
    validate(class)
}
