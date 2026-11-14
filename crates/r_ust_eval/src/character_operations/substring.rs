use crate::*;

pub(crate) fn call_substr(args: &[EvaluatedArgument], substring: bool) -> Result<Sexp, EvalError> {
    let original = positional(args, 0, if substring { "text" } else { "x" })?;
    let start = positional(args, 1, if substring { "first" } else { "start" })?;
    let stop = if substring {
        optional_positional_numeric_vector(args, 2, &["last"], vec![1_000_000.0])?
    } else {
        numeric_arg_vec(args, 2, &["stop"], None)?
    };
    let text = string_values(original)?;
    let start = numeric_values(start)?;
    let len = recycled_len(&[text.len(), start.len(), stop.len()]);
    let mut out = Vec::with_capacity(len);
    for index in 0..len {
        let value = match (
            &text[index % text.len()],
            start[index % start.len()],
            stop[index % stop.len()],
        ) {
            (Some(value), start, stop) if start.is_finite() && stop.is_finite() => Some(
                substring_chars(value, saturating_index(start), saturating_index(stop))?,
            ),
            _ => None,
        };
        out.push(value);
    }
    Ok(character_attributes::with_values(original, out))
}

fn substring_chars(value: &str, start: isize, stop: isize) -> Result<String, EvalError> {
    let start = start.max(1);
    if stop < start {
        return Ok(String::new());
    }
    let start =
        usize::try_from(start).map_err(|_| EvalError::Domain("substring start overflow".into()))?;
    let stop =
        usize::try_from(stop).map_err(|_| EvalError::Domain("substring stop overflow".into()))?;
    Ok(value
        .chars()
        .skip(start - 1)
        .take(stop - start + 1)
        .collect())
}

use super::indices::saturating_index;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finite_index_limits_preserve_existing_saturation_without_panics() -> Result<(), EvalError> {
        assert_eq!(saturating_index(f64::MAX), isize::MAX);
        assert_eq!(saturating_index(-f64::MAX), isize::MIN);
        assert_eq!(saturating_index(2.9), 2);
        assert_eq!(substring_chars("abc", isize::MIN, isize::MAX)?, "abc");
        assert_eq!(substring_chars("abc", isize::MAX, isize::MAX)?, "");
        assert_eq!(substring_chars("abc", 1, -1)?, "");
        Ok(())
    }
}
