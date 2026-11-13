use std::collections::BTreeMap;

use super::{
    coerce_logical_values, format_number, value_len, EvalError, EvaluatedArgument, ListElement,
    OutputEvent, RuntimeState, Sexp,
};

const MANDATORY: &[&str] = &[
    "prompt",
    "continue",
    "expressions",
    "width",
    "deparse.cutoff",
    "digits",
    "echo",
    "quiet",
    "verbose",
    "check.bounds",
    "keep.source",
    "keep.source.pkgs",
    "keep.parse.data",
    "keep.parse.data.pkgs",
    "warning.length",
    "nwarnings",
    "OutDec",
    "CBoundsCheck",
    "matprod",
    "PCRE_study",
    "PCRE_use_JIT",
    "PCRE_limit_recursion",
    "rl_word_breaks",
    "max.contour.segments",
    "warnPartialMatchDollar",
    "warnPartialMatchArgs",
    "warnPartialMatchAttr",
    "showWarnCalls",
    "showErrorCalls",
    "showNCalls",
    "browserNLdisabled",
    "warn",
    "max.print",
    "show.error.messages",
    "scipen",
];

pub(super) fn defaults() -> BTreeMap<String, Sexp> {
    let mut values = BTreeMap::new();
    for (name, value) in [
        ("prompt", "> "),
        ("continue", "+ "),
        ("OutDec", "."),
        ("matprod", "default"),
    ] {
        values.insert(name.into(), Sexp::String(vec![Some(value.into())]));
    }
    for (name, value) in [
        ("expressions", 5000),
        ("width", 80),
        ("digits", 7),
        ("deparse.cutoff", 60),
        ("warning.length", 1000),
        ("nwarnings", 50),
        ("max.contour.segments", 25000),
        ("showNCalls", 50),
        ("warn", 0),
        ("max.print", 99999),
        ("scipen", 0),
    ] {
        values.insert(name.into(), Sexp::Integer(vec![Some(value)]));
    }
    for (name, value) in [
        ("echo", false),
        ("quiet", false),
        ("verbose", false),
        ("check.bounds", false),
        ("keep.source", false),
        ("keep.source.pkgs", false),
        ("keep.parse.data", true),
        ("keep.parse.data.pkgs", false),
        ("CBoundsCheck", false),
        ("PCRE_study", false),
        ("PCRE_use_JIT", true),
        ("warnPartialMatchDollar", false),
        ("warnPartialMatchArgs", false),
        ("warnPartialMatchAttr", false),
        ("showWarnCalls", false),
        ("showErrorCalls", true),
        ("browserNLdisabled", false),
        ("show.error.messages", true),
        ("stringsAsFactors", false),
    ] {
        values.insert(name.into(), Sexp::Logical(vec![Some(value)]));
    }
    values.insert("PCRE_limit_recursion".into(), Sexp::Logical(vec![None]));
    values
}

impl RuntimeState {
    /// Reads this runtime's option store without evaluating R code or changing visibility.
    pub fn get_option(&self, name: &str) -> Option<&Sexp> {
        self.options.get(name)
    }

    /// Sets or removes an option using R's validation, returning the previous value.
    pub fn set_option(&mut self, name: &str, value: Sexp) -> Result<Sexp, EvalError> {
        valid_name(name)?;
        if matches!(value, Sexp::Nil) {
            if MANDATORY.contains(&name) {
                return Err(EvalError::Domain(format!(
                    "option '{name}' cannot be deleted"
                )));
            }
            return Ok(self.options.remove(name).unwrap_or(Sexp::Nil));
        }
        let value = self.normalize_option(name, value)?;
        Ok(self.options.insert(name.into(), value).unwrap_or(Sexp::Nil))
    }

    pub(super) fn call_options(&mut self, args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
        if args.is_empty() {
            self.visible = true;
            // shortcut: byte ordering; use R locale collation when the runtime gains locales.
            return Ok(Sexp::List(
                self.options
                    .iter()
                    .map(|(name, value)| ListElement {
                        name: Some(name.clone()),
                        value: value.clone(),
                    })
                    .collect(),
            ));
        }
        let mut arguments: Vec<ListElement> = args
            .iter()
            .map(|arg| ListElement {
                name: arg.name.clone(),
                value: arg.value.clone(),
            })
            .collect();
        if args.len() == 1 && args[0].name.is_none() {
            match &args[0].value {
                Sexp::Nil => arguments.clear(),
                Sexp::List(values)
                | Sexp::DataFrame {
                    columns: values, ..
                } => {
                    if !values.is_empty() && values.iter().all(|value| value.name.is_none()) {
                        return Err(EvalError::Type("list argument has no valid names".into()));
                    }
                    arguments = values.clone();
                }
                Sexp::Vector(values) if values.is_empty() => arguments.clear(),
                Sexp::Vector(_) => {
                    return Err(EvalError::Type("list argument has no valid names".into()))
                }
                _ => {}
            }
        }
        let mut result = Vec::with_capacity(arguments.len());
        let mut visible = false;
        for argument in arguments {
            let (name, previous) = if let Some(name) = argument.name.filter(|name| !name.is_empty())
            {
                let previous = self.set_option(&name, argument.value)?;
                (name, previous)
            } else {
                let name = option_query(&argument.value, false)?;
                if name == "par.ask.default" {
                    return Err(obsolete_ask());
                }
                visible = true;
                (
                    name.into(),
                    self.get_option(name).cloned().unwrap_or(Sexp::Nil),
                )
            };
            result.push(ListElement {
                name: Some(name),
                value: previous,
            });
        }
        self.visible = visible;
        Ok(Sexp::List(result))
    }

    fn normalize_option(&mut self, name: &str, value: Sexp) -> Result<Sexp, EvalError> {
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
                || (name == "showNCalls" && value_len(&value) != 1)
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
            "editor" if matches!(atomic_data(&value), Sexp::String(_)) => {
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
                ) || value_len(&value) != 1
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
                if !matches!(atomic_data(&value), Sexp::String(values) if values.len() == 2) {
                    return Err(invalid());
                }
                Ok(value)
            }
            "OutDec" | "rl_word_breaks" => {
                let Sexp::String(values) = atomic_data(&value) else {
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

fn atomic_data(value: &Sexp) -> &Sexp {
    match value {
        Sexp::Matrix { data, .. } => data,
        other => other,
    }
}

pub(super) fn option_integer(value: &Sexp) -> Option<i32> {
    super::coercion::silent_integer(value)
}

fn option_text(value: &Sexp) -> Option<String> {
    match atomic_data(value) {
        Sexp::String(values) => values.first()?.clone(),
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

pub(super) fn option_query(value: &Sexp, scalar: bool) -> Result<&str, EvalError> {
    let Sexp::String(values) = atomic_data(value) else {
        return Err(EvalError::Type("'x' must be a character string".into()));
    };
    if values.is_empty() || (scalar && values.len() != 1) {
        return Err(EvalError::Type("'x' must be a character string".into()));
    }
    let name = values[0].as_deref().unwrap_or("NA");
    valid_name(name)?;
    Ok(name)
}

fn valid_name(name: &str) -> Result<(), EvalError> {
    if name.is_empty() || name.contains('\0') {
        return Err(EvalError::Domain(
            "attempt to use zero-length or invalid variable name".into(),
        ));
    }
    Ok(())
}

fn obsolete_ask() -> EvalError {
    EvalError::Domain("\"par.ask.default\" has been replaced by \"device.ask.default\"".into())
}
