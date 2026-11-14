use r_ust_sexp::{is_na_real, Sexp, NA_REAL};

use super::strings::{self, BlankPolicy};
use super::warnings::Warnings;
use crate::{factor_numbers, numeric_values, real_to_integer, EvalError};

pub(super) fn reals(value: &Sexp, policy: BlankPolicy) -> Result<(Vec<f64>, Warnings), EvalError> {
    let mut warnings = Warnings::default();
    let values = match value {
        Sexp::Environment(_) => return Err(crate::environment_values::coercion_error("double")),
        Sexp::Matrix { data, .. } => return reals(data, policy),
        Sexp::String(values) => values
            .iter()
            .map(|value| strings::real(value.as_deref(), policy, &mut warnings))
            .collect(),
        other => numeric_values(other)?,
    };
    Ok((values, warnings))
}

pub(super) fn integers(
    value: &Sexp,
    policy: BlankPolicy,
) -> Result<(Vec<Option<i32>>, Warnings), EvalError> {
    let mut warnings = Warnings::default();
    let values = match value {
        Sexp::Environment(_) => return Err(crate::environment_values::coercion_error("integer")),
        Sexp::Matrix { data, .. } => return integers(data, policy),
        Sexp::String(values) => values
            .iter()
            .map(|value| {
                let number = strings::real(value.as_deref(), policy, &mut warnings);
                integer(number, true, &mut warnings)
            })
            .collect(),
        other => numeric_values(other)?
            .into_iter()
            .map(|value| integer(value, false, &mut warnings))
            .collect(),
    };
    Ok((values, warnings))
}

fn integer(value: f64, from_string: bool, warnings: &mut Warnings) -> Option<i32> {
    if value.is_nan() {
        // shortcut: valid string NaN casts to zero on arm64 R; other C cast profiles need separate evidence.
        return (from_string && !is_na_real(value)).then_some(0);
    }
    if value >= 2147483648.0 || value <= -2147483648.0 {
        warnings.range = true;
        return None;
    }
    real_to_integer(value)
}

pub(super) fn scalar_integer(value: &Sexp, policy: BlankPolicy) -> (Option<i32>, Warnings) {
    let (number, mut warnings) = scalar_real(value, policy);
    let mut data = value;
    while let Sexp::Matrix { data: inner, .. } = data {
        data = inner;
    }
    let result = integer(number, matches!(data, Sexp::String(_)), &mut warnings);
    (result, warnings)
}

pub(super) fn scalar_real(value: &Sexp, policy: BlankPolicy) -> (f64, Warnings) {
    let mut warnings = Warnings::default();
    let number = match value {
        Sexp::Matrix { data, .. } => return scalar_real(data, policy),
        Sexp::String(values) => strings::real(
            values.first().and_then(|value| value.as_deref()),
            policy,
            &mut warnings,
        ),
        Sexp::Real(values) => values.first().copied().unwrap_or(NA_REAL),
        Sexp::Integer(values) => values.first().copied().flatten().map_or(NA_REAL, f64::from),
        Sexp::Logical(values) => values
            .first()
            .copied()
            .flatten()
            .map_or(NA_REAL, |value| f64::from(u8::from(value))),
        Sexp::Factor { codes, .. } => factor_numbers(&codes[..codes.len().min(1)])
            .first()
            .copied()
            .unwrap_or(NA_REAL),
        Sexp::Nil
        | Sexp::Vector(_)
        | Sexp::List(_)
        | Sexp::DataFrame { .. }
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Environment(_)
        | Sexp::Primitive(_)
        | Sexp::Function { .. }
        | Sexp::If { .. }
        | Sexp::For { .. }
        | Sexp::While { .. }
        | Sexp::Repeat { .. }
        | Sexp::Break
        | Sexp::Next
        | Sexp::Block(_)
        | Sexp::Subset { .. }
        | Sexp::Subset2D { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::InternalCall { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => NA_REAL,
    };
    (number, warnings)
}
