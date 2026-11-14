use crate::{logical_values, numeric_scalar, EvalError, ListElement, Sexp};

pub(super) fn validate(controls: &[ListElement]) -> Result<bool, EvalError> {
    let [digits, quote, na, gap, right, max, width, source, rest @ ..] = controls else {
        return Err(EvalError::Type(
            "invalid print.default control pairlist".into(),
        ));
    };
    if !matches!(digits.value, Sexp::Nil) {
        let digits = numeric_scalar(&digits.value, "digits")?;
        if !digits.is_finite() || !(1.0..23.0).contains(&digits) {
            return Err(EvalError::Domain("invalid printing digits".into()));
        }
        return Err(EvalError::NotImplemented("print.default digits"));
    }
    let quote = boolean(&quote.value, "quote")?;
    if !matches!(na.value, Sexp::Nil) {
        if na
            .value
            .character_values()
            .is_none_or(|values| values.is_empty())
        {
            return Err(EvalError::Type("invalid 'na.print' specification".into()));
        }
        return Err(EvalError::NotImplemented("print.default na.print"));
    }
    if !matches!(gap.value, Sexp::Nil) {
        let gap = numeric_scalar(&gap.value, "print.gap")?;
        if !gap.is_finite() || gap.trunc() < 0.0 {
            return Err(EvalError::Domain(
                "'gap' must be non-negative integer".into(),
            ));
        }
        if gap.trunc() > 1024.0 {
            return Err(EvalError::Domain(
                "'print.gap' must be less than 1024".into(),
            ));
        }
        return Err(EvalError::NotImplemented("print.default print.gap"));
    }
    if boolean(&right.value, "right")? {
        return Err(EvalError::NotImplemented("print.default right"));
    }
    if !matches!(max.value, Sexp::Nil) {
        let max = numeric_scalar(&max.value, "max")?;
        if !max.is_finite() || max.trunc() < 0.0 || max.trunc() > f64::from(i32::MAX) {
            return Err(EvalError::Domain("invalid 'max' argument".into()));
        }
        return Err(EvalError::NotImplemented("print.default max"));
    }
    if !matches!(width.value, Sexp::Nil) {
        let width = numeric_scalar(&width.value, "width")?;
        if !width.is_finite() || !(10.0..10001.0).contains(&width) {
            return Err(EvalError::Domain("invalid printing width".into()));
        }
        return Err(EvalError::NotImplemented("print.default width"));
    }
    if !boolean(&source.value, "useSource")? {
        return Err(EvalError::NotImplemented("print.default useSource"));
    }
    if !rest.is_empty() {
        return Err(EvalError::NotImplemented("print.default extra controls"));
    }
    Ok(quote)
}

fn boolean(value: &Sexp, name: &str) -> Result<bool, EvalError> {
    logical_values(value)?
        .first()
        .copied()
        .flatten()
        .ok_or_else(|| EvalError::Domain(format!("invalid '{name}' argument")))
}
