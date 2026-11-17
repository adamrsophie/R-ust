use super::*;

#[cfg(test)]
mod tests;

pub(super) fn string_values(value: &Sexp) -> Result<Vec<Option<String>>, EvalError> {
    crate::ordinary_cycles::check(value, "cyclic ordinary value coercion")?;
    let projection = crate::coercion::storage::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Environment(_) => Err(environment_values::coercion_error("character")),
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::NamedList {
            elements: values, ..
        }
        | Sexp::List(values) => values
            .iter()
            .map(|element| message_helpers::list_character(&element.value).map(Some))
            .collect(),
        Sexp::Vector(values) => values
            .iter()
            .map(|value| message_helpers::list_character(value).map(Some))
            .collect(),
        Sexp::Raw(values) => Ok(values.iter().map(|v| Some(format!("{v:02x}"))).collect()),
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Symbol(name) => Ok(vec![Some(name.clone())]),
        Sexp::Matrix { data, .. } => string_values(data),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => Ok(values.clone()),
        Sexp::Real(values) => Ok(values
            .iter()
            .map(|value| {
                if is_na_real(*value) {
                    None
                } else {
                    Some(format_number(*value))
                }
            })
            .collect()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .map(|value| value.map(|value| value.to_string()))
            .collect()),
        Sexp::Logical(values) => Ok(values
            .iter()
            .map(|value| {
                value.map(|value| {
                    if value {
                        "TRUE".to_string()
                    } else {
                        "FALSE".to_string()
                    }
                })
            })
            .collect()),
        Sexp::Factor { codes, levels, .. } => Ok(factor_strings(codes, levels)),
        other => Err(EvalError::Type(format!(
            "expected atomic value, found {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn coerce_logical_values(value: &Sexp) -> Result<Vec<Option<bool>>, EvalError> {
    let projection = crate::coercion::storage::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Environment(_) => Err(environment_values::coercion_error("logical")),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => Ok(values
            .iter()
            .map(|value| match value.as_deref() {
                Some("TRUE") | Some("True") | Some("true") | Some("T") => Some(true),
                Some("FALSE") | Some("False") | Some("false") | Some("F") => Some(false),
                _ => None,
            })
            .collect()),
        other => logical_values(other),
    }
}

pub(super) fn numeric_values(value: &Sexp) -> Result<Vec<f64>, EvalError> {
    let projection = coercion::storage::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Raw(values) => Ok(values.iter().map(|v| f64::from(*v)).collect()),
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Matrix { data, .. } => numeric_values(data),
        Sexp::Real(values) => Ok(values.clone()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .map(|value| value.map_or(NA_REAL, f64::from))
            .collect()),
        Sexp::Logical(values) => Ok(values
            .iter()
            .map(|value| match value {
                Some(true) => 1.0,
                Some(false) => 0.0,
                None => NA_REAL,
            })
            .collect()),
        Sexp::Factor { codes, .. } => Ok(factor_numbers(codes)),
        other => Err(EvalError::Type(format!(
            "expected numeric value, found {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn logical_values(value: &Sexp) -> Result<Vec<Option<bool>>, EvalError> {
    let projection = coercion::storage::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Raw(values) => Ok(values.iter().map(|v| Some(*v != 0)).collect()),
        Sexp::Nil => Ok(Vec::new()),
        Sexp::Matrix { data, .. } => logical_values(data),
        Sexp::Logical(values) => Ok(values.clone()),
        Sexp::Integer(values) => Ok(values
            .iter()
            .map(|value| value.map(|value| value != 0))
            .collect()),
        Sexp::Real(values) => Ok(values
            .iter()
            .map(|value| {
                if value.is_nan() {
                    None
                } else {
                    Some(*value != 0.0)
                }
            })
            .collect()),
        other => Err(EvalError::Type(format!(
            "expected logical value, found {:?}",
            other.sexptype()
        ))),
    }
}

pub(super) fn string_scalar(value: &Sexp, label: &str) -> Result<String, EvalError> {
    let projection = coercion::storage::read(value)?;
    let value = projection.as_ref();
    let Some(values) = value.character_values() else {
        return Err(EvalError::Type(format!("{label} must be a string")));
    };
    values
        .first()
        .cloned()
        .flatten()
        .ok_or_else(|| EvalError::Arity(format!("{label} has length 0")))
}
