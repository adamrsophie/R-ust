use std::collections::BTreeMap;

use super::{EvalError, EvaluatedArgument, ListElement, RuntimeState, Sexp};

mod validation;
use validation::atomic_data;

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
            let projection = crate::ordinary_values::read(&args[0].value)?;
            match projection.as_ref() {
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
}

pub(super) fn option_integer(value: &Sexp) -> Option<i32> {
    super::coercion::silent_integer(value)
}

pub(super) fn option_query(value: &Sexp, scalar: bool) -> Result<&str, EvalError> {
    let (Sexp::String(values) | Sexp::AttributedCharacter { values, .. }) = atomic_data(value)
    else {
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
