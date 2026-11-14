use super::*;

pub(super) fn string_values(value: &Sexp) -> Result<Vec<Option<String>>, EvalError> {
    crate::ordinary_cycles::check(value, "cyclic ordinary value coercion")?;
    let projection = crate::ordinary_values::read(value)?;
    let value = projection.as_ref();
    match value {
        Sexp::Environment(_) => Err(environment_values::coercion_error("character")),
        Sexp::ClassedList {
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
        Sexp::Nil => Ok(Vec::new()),
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
