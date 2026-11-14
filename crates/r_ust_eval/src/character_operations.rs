use super::*;
mod indices;
mod substring;
pub(super) use substring::call_substr;

pub(super) fn call_case(args: &[EvaluatedArgument], upper: bool) -> Result<Sexp, EvalError> {
    let value = positional(args, 0, "x")?;
    Ok(character_attributes::with_values(
        value,
        string_values(value)?
            .into_iter()
            .map(|value| {
                value.map(|value| {
                    if upper {
                        value.to_uppercase()
                    } else {
                        value.to_lowercase()
                    }
                })
            })
            .collect(),
    ))
}

pub(super) fn call_sub_gsub(args: &[EvaluatedArgument], global: bool) -> Result<Sexp, EvalError> {
    let pattern = string_scalar(positional(args, 0, "pattern")?, "pattern")?;
    let replacement = string_scalar(positional(args, 1, "replacement")?, "replacement")?;
    let x = string_values(positional(args, 2, "x")?)?;
    let ignore_case = named_bool_arg(args, &["ignore.case"], false)?;
    let fixed = named_bool_arg(args, &["fixed"], false)?;
    Ok(character_attributes::with_values(
        positional(args, 2, "x")?,
        x.into_iter()
            .map(|value| {
                value.map(|value| {
                    replace_pattern(&pattern, &replacement, &value, ignore_case, fixed, global)
                })
            })
            .collect(),
    ))
}

pub(super) fn call_trimws(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = string_values(positional(args, 0, "x")?)?;
    let which = optional_string_arg(args, 1, &["which"], "both")?;
    let trim = match which.as_str() {
        "both" => str::trim,
        "left" => str::trim_start,
        "right" => str::trim_end,
        _ => {
            return Err(EvalError::Domain(
                "argument which must be \"both\", \"left\", or \"right\"".to_string(),
            ))
        }
    };
    Ok(character_attributes::with_values(
        positional(args, 0, "x")?,
        x.into_iter()
            .map(|value| value.map(|value| trim(&value).to_string()))
            .collect(),
    ))
}

pub(super) fn call_chartr(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let old = string_scalar(positional(args, 0, "old")?, "old")?;
    let new = string_scalar(positional(args, 1, "new")?, "new")?;
    let x = string_values(positional(args, 2, "x")?)?;
    let old_chars = old.chars().collect::<Vec<_>>();
    let new_chars = new.chars().collect::<Vec<_>>();
    if old_chars.len() != new_chars.len() {
        return Err(EvalError::Domain(
            "old and new must have the same length".to_string(),
        ));
    }
    Ok(character_attributes::with_values(
        positional(args, 2, "x")?,
        x.into_iter()
            .map(|value| {
                value.map(|value| {
                    value
                        .chars()
                        .map(|ch| {
                            old_chars
                                .iter()
                                .position(|old| *old == ch)
                                .and_then(|position| new_chars.get(position).copied())
                                .unwrap_or(ch)
                        })
                        .collect()
                })
            })
            .collect(),
    ))
}

pub(super) fn call_strtrim(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    let x = string_values(positional(args, 0, "x")?)?;
    let width = numeric_arg_scalar(args, 1, &["width"], None)?;
    if width.is_nan() || !width.is_finite() || width < 0.0 {
        return Err(EvalError::Domain("invalid width argument".to_string()));
    }
    let width = indices::saturating_length(width);
    Ok(character_attributes::with_values(
        positional(args, 0, "x")?,
        x.into_iter()
            .map(|value| value.map(|value| value.chars().take(width).collect()))
            .collect(),
    ))
}
