// Adapted from R 4.6.0 src/main/summary.c.
// Copyright (C) 1997-2025 The R Core Team
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka
// SPDX-License-Identifier: GPL-2.0-or-later

mod arithmetic;
mod input;
mod integer_sum;
mod product;
mod sum;

use super::{coerce_logical_values, typeof_value, value_len, EvalError, EvaluatedArgument, Sexp};

fn arguments(args: &[EvaluatedArgument], operation: &str) -> Result<bool, EvalError> {
    let mut flags = args
        .iter()
        .filter(|arg| arg.name.as_deref() == Some("na.rm"));
    let flag = flags.next();
    if flags.next().is_some() {
        return Err(EvalError::Arity(
            "formal argument \"na.rm\" matched by multiple actual arguments".into(),
        ));
    }
    let first = args
        .iter()
        .find(|arg| arg.name.as_deref() != Some("na.rm"))
        .or(flag);
    if let Some(EvaluatedArgument {
        value: Sexp::Factor { ordered, .. },
        ..
    }) = first
    {
        let message = if *ordered {
            format!("'{operation}' not defined for ordered factors")
        } else {
            // shortcut: use straight quotes, locale-aware condition rendering remains separate.
            format!("'{operation}' not meaningful for factors")
        };
        return Err(EvalError::Type(message));
    }
    let Some(flag) = flag else {
        return Ok(false);
    };
    let value = storage(&flag.value);
    if matches!(
        typeof_value(value),
        "logical" | "integer" | "double" | "character"
    ) && value_len(value)? > 1
    {
        return Err(EvalError::Domain(format!(
            "'length = {}' in coercion to 'logical(1)'",
            value_len(value)?
        )));
    }
    let logical = match value {
        Sexp::Factor { codes, .. } => codes.first().copied().flatten().map(|code| code != 0),
        _ => coerce_logical_values(value)
            .map_err(|_| EvalError::Domain("NA in coercion to boolean".into()))?
            .first()
            .copied()
            .flatten(),
    };
    logical.ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))
}

fn storage(value: &Sexp) -> &Sexp {
    match value {
        Sexp::Matrix { data, .. } => storage(data),
        other => other,
    }
}

pub(super) fn sum(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    sum::evaluate(args, arguments(args, "sum")?)
}

pub(super) fn product(args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
    product::evaluate(args, arguments(args, "prod")?)
}
