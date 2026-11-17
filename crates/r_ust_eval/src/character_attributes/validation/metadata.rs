use crate::{list_attributes::engine, EvalError, ListElement, Sexp};

pub(super) fn validate(object: &Sexp, attributes: &[ListElement]) -> Result<(), EvalError> {
    let size = engine::length(object)?.unwrap_or(0);
    let mut dims = None;
    for attribute in attributes {
        let name = attribute.name.as_deref().unwrap_or("");
        let plain = engine::payload(&attribute.value)?;
        match name {
            "names" => {
                let names = plain.character_values().ok_or_else(|| invalid(name))?;
                if names.len() != size {
                    return Err(invalid(name));
                }
            }
            "class" => {
                super::check_class(&plain)?;
                if plain.character_values().is_some_and(|classes| {
                    classes
                        .iter()
                        .any(|class| class.as_deref() == Some("factor"))
                }) && object.sexptype() != r_ust_sexp::SexpType::Integer
                {
                    return Err(invalid(name));
                }
            }
            "dim" => dims = Some(dimensions(&plain, size)?),
            "row.names" if !matches!(plain, Sexp::Integer(_) | Sexp::String(_)) => {
                return Err(invalid(name));
            }
            "comment" if plain.character_values().is_none() => return Err(invalid(name)),
            "tsp" => match &plain {
                Sexp::Integer(values) if values.len() == 3 => {}
                Sexp::Real(values) if values.len() == 3 => {}
                _ => return Err(invalid(name)),
            },
            _ => {}
        }
    }
    if let Some(value) = attributes
        .iter()
        .find(|attribute| attribute.name.as_deref() == Some("dimnames"))
    {
        dimnames(
            &value.value,
            dims.as_deref().ok_or_else(|| invalid("dimnames"))?,
        )?;
    }
    Ok(())
}

fn dimensions(value: &Sexp, size: usize) -> Result<Vec<usize>, EvalError> {
    let Sexp::Integer(values) = value else {
        return Err(invalid("dim"));
    };
    if values.is_empty() {
        return Err(invalid("dim"));
    }
    let dimensions = values
        .iter()
        .map(|value| {
            value
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| invalid("dim"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if dimensions
        .iter()
        .try_fold(1usize, |product, dim| product.checked_mul(*dim))
        != Some(size)
    {
        return Err(invalid("dim"));
    }
    Ok(dimensions)
}

fn dimnames(value: &Sexp, dimensions: &[usize]) -> Result<(), EvalError> {
    let Sexp::Vector(values) = engine::payload(value)? else {
        return Err(invalid("dimnames"));
    };
    if values.len() != dimensions.len() {
        return Err(invalid("dimnames"));
    }
    for (value, size) in values.iter().zip(dimensions) {
        let value = engine::payload(value)?;
        if matches!(value, Sexp::Nil) {
            continue;
        }
        let names = value
            .character_values()
            .ok_or_else(|| invalid("dimnames"))?;
        if names.len() != *size {
            return Err(invalid("dimnames"));
        }
    }
    Ok(())
}

fn invalid(name: &str) -> EvalError {
    EvalError::Type(format!("invalid stored '{name}' attribute"))
}
