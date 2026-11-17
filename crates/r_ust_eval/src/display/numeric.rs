use crate::{EvalError, Sexp};

pub(super) fn tokens(
    value: &Sexp,
    quote: bool,
) -> Result<(Vec<String>, bool, &'static str), EvalError> {
    Ok(match value {
        Sexp::Logical(values) => (
            values
                .iter()
                .map(|v| {
                    match v {
                        Some(true) => "TRUE",
                        Some(false) => "FALSE",
                        None => "NA",
                    }
                    .into()
                })
                .collect(),
            false,
            "logical(0)",
        ),
        Sexp::Integer(values) => (
            values
                .iter()
                .map(|v| v.map_or_else(|| "NA".into(), |v| v.to_string()))
                .collect(),
            false,
            "integer(0)",
        ),
        Sexp::Real(values) => (reals(values)?, false, "numeric(0)"),
        Sexp::Raw(values) => (
            values.iter().map(|v| format!("{v:02x}")).collect(),
            false,
            "raw(0)",
        ),
        Sexp::String(values) => (
            values
                .iter()
                .map(|v| match v {
                    Some(v) => {
                        crate::encode_string::encode(v, if quote { Some('"') } else { None })
                    }
                    None => if quote { "NA" } else { "<NA>" }.into(),
                })
                .collect(),
            true,
            "character(0)",
        ),
        _ => return Err(EvalError::Type("invalid atomic print payload".into())),
    })
}

fn reals(values: &[f64]) -> Result<Vec<String>, EvalError> {
    let mut left_max = i32::MIN;
    let mut left_min = i32::MAX;
    let mut signed_left = 0;
    let mut right = 0;
    let mut significant = 1;
    let mut negative = 0;
    for value in values.iter().filter(|v| v.is_finite()) {
        let (power, digits, widens) = scientific(*value)?;
        let left = power + 1 - i32::from(widens);
        let sign = i32::from(*value < 0.0);
        negative = negative.max(sign);
        left_max = left_max.max(left);
        left_min = left_min.min(left);
        signed_left = signed_left.max(sign + left.max(1));
        right = right.max(digits - left);
        significant = significant.max(digits);
    }
    if left_max < 0 {
        signed_left = 1 + negative;
    }
    let fixed_width = signed_left + right + i32::from(right != 0);
    let exponent_digits = if left_max > 100 || left_min <= -99 {
        3
    } else {
        2
    };
    let scientific_width =
        negative + i32::from(significant > 1) + significant + 2 + exponent_digits;
    let exponential = fixed_width > scientific_width;
    let precision = usize::try_from(if exponential { significant - 1 } else { right })
        .map_err(|_| EvalError::Type("invalid numeric print precision".into()))?;
    values
        .iter()
        .map(|value| {
            if !value.is_finite() {
                return Ok(crate::format_number(*value));
            }
            let value = if *value == 0.0 { 0.0 } else { *value };
            if exponential {
                let raw = format!("{value:.precision$e}");
                let (mantissa, exponent) = raw
                    .split_once('e')
                    .ok_or_else(|| EvalError::Type("numeric exponent".into()))?;
                let exponent: i32 = exponent
                    .parse()
                    .map_err(|_| EvalError::Type("numeric exponent".into()))?;
                Ok(format!("{mantissa}e{exponent:+03}"))
            } else {
                Ok(format!("{value:.precision$}"))
            }
        })
        .collect()
}

fn scientific(value: f64) -> Result<(i32, i32, bool), EvalError> {
    if value == 0.0 {
        return Ok((0, 1, false));
    }
    let text = format!("{:.6e}", value.abs());
    let (mantissa, exponent) = text
        .split_once('e')
        .ok_or_else(|| EvalError::Type("numeric exponent".into()))?;
    let power: i32 = exponent
        .parse()
        .map_err(|_| EvalError::Type("numeric exponent".into()))?;
    let digits = mantissa
        .trim_end_matches('0')
        .chars()
        .filter(|c| *c != '.')
        .count();
    let fuzz = 0.5 / 10.0_f64.powi((7 - power).clamp(0, 22));
    let widens = power > 0 && power <= 22 && value.abs() < 10.0_f64.powi(power) - fuzz;
    Ok((
        power,
        i32::try_from(digits).map_err(|_| EvalError::Type("numeric precision".into()))?,
        widens,
    ))
}
