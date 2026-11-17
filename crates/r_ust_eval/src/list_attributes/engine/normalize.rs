use super::*;

pub(super) fn value(object: &Sexp, update: (&str, &Sexp)) -> Result<Sexp, EvalError> {
    let (name, value) = update;
    if matches!(value, Sexp::Nil) {
        return Ok(Sexp::Nil);
    }
    if matches!(object, Sexp::Nil) {
        return Err(EvalError::Domain(
            "attempt to set an attribute on NULL".into(),
        ));
    }
    match name {
        "names" => names(object, value),
        "class" => class(object, value),
        "dim" => dimensions::dims(object, value),
        "dimnames" => dimensions::names(object, value),
        "row.names" => rows::normalize(value),
        "comment" => {
            let plain = payload(value)?;
            let Some(strings) = plain.character_values() else {
                return Err(EvalError::Type(
                    "attempt to set invalid 'comment' attribute".into(),
                ));
            };
            Ok(if strings.is_empty() {
                Sexp::Nil
            } else {
                value.clone()
            })
        }
        "tsp" => tsp(object, value, f64::NAN),
        _ => Ok(value.clone()),
    }
}

fn class(object: &Sexp, value: &Sexp) -> Result<Sexp, EvalError> {
    let plain = payload(value)?;
    let Some(values) = plain.character_values() else {
        return Err(EvalError::Type(
            "attempt to set invalid 'class' attribute".into(),
        ));
    };
    if values.iter().any(|name| name.as_deref() == Some("factor"))
        && object.sexptype() != r_ust_sexp::SexpType::Integer
    {
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

fn names(object: &Sexp, value: &Sexp) -> Result<Sexp, EvalError> {
    let count = length(object)?
        .ok_or_else(|| EvalError::Domain("names() applied to a non-vector".into()))?;
    let plain = payload(value)?;
    let mut names = crate::string_values(&plain)?;
    if names.len() > count {
        return Err(EvalError::Domain(format!(
            "'names' attribute [{}] must be the same length as the vector [{}]",
            names.len(),
            count
        )));
    }
    if names.len() == count && plain.character_values().is_some() {
        return Ok(value.clone());
    }
    let attrs = replacements::resized_attributes(value, (names.len(), count))?;
    names.resize(count, None);
    Ok(if attrs.is_empty() {
        Sexp::String(names)
    } else {
        Sexp::AttributedCharacter {
            values: names,
            attributes: attrs,
        }
    })
}

pub(super) fn tsp(object: &Sexp, value: &Sexp, tolerance: f64) -> Result<Sexp, EvalError> {
    let plain = payload(value)?;
    if !matches!(plain, Sexp::Integer(_) | Sexp::Real(_)) || crate::value_len(&plain)? != 3 {
        return Err(EvalError::Type(
            "'tsp' attribute must be numeric of length three".into(),
        ));
    }
    let numbers = crate::numeric_values(&plain)?;
    let frequency = numbers[2];
    if frequency <= 0.0 {
        return Err(EvalError::Domain(
            "invalid time series parameters specified (0)".into(),
        ));
    }
    let attrs = stored(object)?;
    let n = match find(&attrs, "dim") {
        Some(dim) => crate::coerce_integer_values(&payload(dim)?)?
            .first()
            .copied()
            .flatten()
            .and_then(|n| usize::try_from(n).ok())
            .unwrap_or(0),
        None => length(object)?.unwrap_or(1),
    };
    if n == 0 {
        return Err(EvalError::Domain(
            "cannot assign 'tsp' to zero-length vector".into(),
        ));
    }
    let n = f64::from(u32::try_from(n).map_err(|_| EvalError::NotImplemented("long time series"))?);
    if (numbers[1] - numbers[0] - (n - 1.0) / frequency).abs() > tolerance {
        return Err(EvalError::Domain(
            "invalid time series parameters specified (1)".into(),
        ));
    }
    Ok(Sexp::Real(numbers))
}
