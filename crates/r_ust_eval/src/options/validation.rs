use super::{obsolete_ask, option_integer};
use crate::{
    coerce_logical_values, format_number, value_len, EvalError, OutputEvent, RuntimeState, Sexp,
};

impl RuntimeState {
    pub(super) fn normalize_option(&mut self, name: &str, value: Sexp) -> Result<Sexp, EvalError> {
        let invalid = || EvalError::Domain(format!("invalid value for '{name}'"));
        let range = match name {
            "width" => Some((10, 10000)),
            "digits" => Some((1, 22)),
            "expressions" => Some((25, 500000)),
            "warning.length" => Some((100, 8170)),
            "max.print" | "nwarnings" => Some((1, i32::MAX)),
            "max.contour.segments" => Some((0, i32::MAX)),
            "showNCalls" => Some((30, 500)),
            _ => None,
        };
        if let Some((min, max)) = range {
            let integer = self.scalar_integer_with_warning(&value)?;
            if !integer.is_some_and(|value| (min..=max).contains(&value))
                || (name == "showNCalls" && value_len(&value)? != 1)
            {
                return Err(if matches!(name, "width" | "digits" | "expressions") {
                    EvalError::Domain(format!("invalid '{name}' parameter, allowed {min}...{max}"))
                } else {
                    invalid()
                });
            }
            return Ok(if name == "warning.length" {
                value
            } else {
                Sexp::Integer(vec![integer])
            });
        }
        match name {
            "deparse.cutoff" => Ok(Sexp::Integer(vec![
                self.scalar_integer_with_warning(&value)?
            ])),
            "prompt" | "continue" => {
                let text = option_text(&value)
                    .filter(|text| !text.is_empty())
                    .ok_or_else(invalid)?;
                Ok(Sexp::String(vec![Some(text)]))
            }
            "editor"
                if matches!(
                    atomic_data(&value),
                    Sexp::String(_) | Sexp::AttributedCharacter { .. }
                ) =>
            {
                let text = option_text(&value)
                    .filter(|text| !text.is_empty())
                    .ok_or_else(invalid)?;
                Ok(Sexp::String(vec![Some(text)]))
            }
            "keep.source"
            | "quiet"
            | "verbose"
            | "stringsAsFactors"
            | "check.bounds"
            | "echo"
            | "show.error.messages"
            | "catch.script.errors"
            | "warnPartialMatchDollar"
            | "warnPartialMatchArgs"
            | "warnPartialMatchAttr"
            | "showWarnCalls"
            | "showErrorCalls"
            | "browserNLdisabled"
            | "CBoundsCheck" => {
                let Sexp::Logical(values) = atomic_data(&value) else {
                    return Err(invalid());
                };
                let [logical] = values.as_slice() else {
                    return Err(invalid());
                };
                if logical.is_none() && !matches!(name, "check.bounds" | "echo") {
                    return Err(invalid());
                }
                if name == "stringsAsFactors"
                    && *logical == Some(true)
                    && !self.warned_strings_as_factors
                {
                    self.emit(OutputEvent::Stderr("Warning: 'options(stringsAsFactors = TRUE)' is deprecated and will be disabled\n".into()))?;
                    self.warned_strings_as_factors = true;
                }
                Ok(Sexp::Logical(vec![*logical]))
            }
            "warn" | "scipen" => {
                if !matches!(
                    atomic_data(&value),
                    Sexp::Integer(_) | Sexp::Real(_) | Sexp::Logical(_)
                ) || value_len(&value)? != 1
                {
                    return Err(invalid());
                }
                let integer = option_integer(&value);
                if name == "warn" {
                    return integer
                        .map(|value| Sexp::Integer(vec![Some(value)]))
                        .ok_or_else(invalid);
                }
                if let Sexp::Real(values) = atomic_data(&value) {
                    if integer.is_none() && !values[0].is_nan() {
                        return Err(invalid());
                    }
                }
                let fixed = integer.map_or(0, |value| value.clamp(-9, 9999));
                if integer != Some(fixed) {
                    self.emit(OutputEvent::Stderr(format!(
                        "Warning: invalid 'scipen' {}, used {fixed}\n",
                        integer.unwrap_or(i32::MIN)
                    )))?;
                }
                Ok(Sexp::Integer(vec![Some(fixed)]))
            }
            "contrasts" => {
                if !matches!(atomic_data(&value), Sexp::String(values) | Sexp::AttributedCharacter { values, .. } if values.len() == 2)
                {
                    return Err(invalid());
                }
                Ok(value)
            }
            "OutDec" | "rl_word_breaks" => {
                let (Sexp::String(values) | Sexp::AttributedCharacter { values, .. }) =
                    atomic_data(&value)
                else {
                    return Err(invalid());
                };
                let [text] = values.as_slice() else {
                    return Err(invalid());
                };
                if name == "OutDec" && text.as_ref().is_none_or(|text| text.chars().count() != 1) {
                    self.emit(OutputEvent::Stderr(
                        "Warning: 'OutDec' must be a string of one character\n".into(),
                    ))?;
                }
                Ok(value)
            }
            "matprod" => {
                if !option_text(&value).is_some_and(|text| {
                    matches!(
                        text.as_str(),
                        "default" | "internal" | "blas" | "default.simd"
                    )
                }) {
                    return Err(invalid());
                }
                Ok(value)
            }
            "PCRE_use_JIT" | "PCRE_limit_recursion" => {
                let logical = coerce_logical_values(&value)?.first().copied().flatten();
                Ok(Sexp::Logical(vec![if name == "PCRE_use_JIT" {
                    Some(logical == Some(true))
                } else {
                    logical
                }]))
            }
            "PCRE_study" => {
                let (result, warn) = if let Sexp::Logical(values) = atomic_data(&value) {
                    let logical = values.first() == Some(&Some(true));
                    (Sexp::Logical(vec![Some(logical)]), logical)
                } else {
                    match self.scalar_integer_with_warning(&value)? {
                        Some(value) if value >= 0 => (Sexp::Integer(vec![Some(value)]), true),
                        _ => (Sexp::Logical(vec![Some(true)]), false),
                    }
                };
                if warn {
                    self.emit(OutputEvent::Stderr(
                        "Warning: 'PCRE_study' has no effect with PCRE2\n".into(),
                    ))?;
                }
                Ok(result)
            }
            "error" if matches!(value, Sexp::Closure { .. }) => {
                Err(EvalError::NotImplemented("option error condition handlers"))
            }
            "error" | "warning.expression" => Err(invalid()),
            "par.ask.default" => Err(obsolete_ask()),
            _ => Ok(value),
        }
    }
}

pub(super) fn atomic_data(value: &Sexp) -> &Sexp {
    match value {
        Sexp::Matrix { data, .. } => data,
        other => other,
    }
}

fn option_text(value: &Sexp) -> Option<String> {
    match atomic_data(value) {
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => values.first()?.clone(),
        Sexp::Real(values) => Some(format_number(*values.first()?)),
        Sexp::Integer(values) => Some(
            values
                .first()?
                .map_or_else(|| "NA".into(), |value| value.to_string()),
        ),
        Sexp::Logical(values) => values
            .first()?
            .map(|value| if value { "TRUE" } else { "FALSE" }.into()),
        Sexp::Factor { codes, .. } => Some(
            codes
                .first()?
                .map_or_else(|| "NA".into(), |value| value.to_string()),
        ),
        _ => None,
    }
}
