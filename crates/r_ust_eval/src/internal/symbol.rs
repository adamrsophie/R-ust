use crate::{EvalError, Sexp};

pub(super) fn convert(value: &Sexp) -> Result<Sexp, EvalError> {
    if matches!(value, Sexp::Symbol(_)) {
        return Ok(value.clone());
    }
    let values = match value {
        Sexp::Logical(_)
        | Sexp::Integer(_)
        | Sexp::Real(_)
        | Sexp::String(_)
        | Sexp::AttributedCharacter { .. } => crate::string_values(value)?,
        Sexp::Factor { .. } => {
            crate::string_values(&Sexp::Integer(crate::coerce_integer_values(value)?))?
        }
        Sexp::Matrix { data, .. } => return convert(data),
        _ => {
            return Err(EvalError::Type(format!(
                "cannot coerce type '{}' to vector of type 'symbol'",
                crate::metadata::typeof_value(value),
            )))
        }
    };
    let value_name = values.first().ok_or_else(|| {
        EvalError::Domain(format!(
            "invalid data of mode '{}' (too short)",
            crate::metadata::typeof_value(value),
        ))
    })?;
    let name = value_name.as_deref().unwrap_or("NA");
    if name.is_empty() {
        return Err(EvalError::Domain(
            "attempt to use zero-length variable name".into(),
        ));
    }
    Ok(Sexp::Symbol(name.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factor_symbols_use_integer_storage_instead_of_levels() {
        for (codes, expected) in [(vec![Some(2), Some(1), None], "2"), (vec![None], "NA")] {
            let value = Sexp::Factor {
                codes,
                levels: vec!["a".into(), "z".into()],
                ordered: false,
            };
            assert_eq!(convert(&value), Ok(Sexp::Symbol(expected.into())));
        }
    }
}
