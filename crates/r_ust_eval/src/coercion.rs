use super::{factor_numbers, numeric_values, real_to_integer, EvalError, Sexp, NA_REAL};

mod runtime;

pub(super) fn coerce_numeric_values(value: &Sexp) -> Result<Vec<f64>, EvalError> {
    match value {
        Sexp::Factor { codes, .. } => Ok(factor_numbers(codes)),
        Sexp::String(values) => Ok(values
            .iter()
            .map(|value| {
                value
                    .as_deref()
                    .and_then(|value| value.trim().parse::<f64>().ok())
                    .unwrap_or(NA_REAL)
            })
            .collect()),
        other => numeric_values(other),
    }
}

pub(super) fn coerce_integer_values(value: &Sexp) -> Result<Vec<Option<i32>>, EvalError> {
    Ok(coerce_numeric_values(value)?
        .into_iter()
        .map(real_to_integer)
        .collect())
}
