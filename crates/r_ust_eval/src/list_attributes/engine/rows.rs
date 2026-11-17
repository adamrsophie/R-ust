use super::*;

pub(super) fn normalize(value: &Sexp) -> Result<Sexp, EvalError> {
    let mut plain = payload(value)?;
    if let Sexp::Real(values) = &plain {
        if values.len() == 2 && values[0].is_nan() {
            plain = Sexp::Integer(crate::coerce_integer_values(&plain)?);
        }
    }
    match plain {
        Sexp::Integer(values) => {
            let compact = values.len() > 2
                && values
                    .iter()
                    .enumerate()
                    .all(|(i, n)| i32::try_from(i + 1).ok() == *n);
            Ok(if compact {
                Sexp::Integer(vec![
                    None,
                    Some(
                        i32::try_from(values.len())
                            .map_err(|_| EvalError::Domain("too many row names".into()))?,
                    ),
                ])
            } else {
                Sexp::Integer(values)
            })
        }
        Sexp::String(_) => Ok(value.clone()),
        _ => Err(EvalError::Type(format!(
            "row names must be 'character' or 'integer', not '{}'",
            crate::metadata::typeof_value(value)
        ))),
    }
}
