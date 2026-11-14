// Adapted from R 4.6.0 src/main/coerce.c (anyNA and do_anyNA).
// Copyright (C) 1997-2025 The R Core Team
// Copyright (C) 2003-2023 The R Foundation
// Copyright (C) 1995,1996 Robert Gentleman, Ross Ihaka
// Copyright 2012 Google Inc. All Rights Reserved.
// SPDX-License-Identifier: GPL-2.0-or-later

use super::{
    coerce_logical_values, typeof_value, value_len, CallArgument, EvalError, EvaluatedArgument,
    FunctionParameter, RuntimeState, Sexp,
};

impl RuntimeState {
    pub(super) fn eval_any_na(&mut self, arguments: &[CallArgument]) -> Result<Sexp, EvalError> {
        let mut values = Vec::new();
        for (index, argument) in arguments.iter().enumerate() {
            if matches!(argument.value, Sexp::MissingArg(_)) {
                return Err(EvalError::Arity(format!("argument {} is empty", index + 1)));
            }
            values.extend(self.eval_arguments(std::slice::from_ref(argument), false)?);
        }
        self.visible = true;
        self.call_any_na(&values)
    }

    pub(super) fn call_any_na(&mut self, args: &[EvaluatedArgument]) -> Result<Sexp, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Err(EvalError::Arity("anyNA takes 1 or 2 arguments".into()));
        }
        let default = Sexp::Logical(vec![Some(false)]);
        if let [argument] = args {
            if let Some(name) = &argument.name {
                if name != "x" {
                    return Err(EvalError::Arity(format!(
                        "supplied argument name '{name}' does not match 'x'"
                    )));
                }
            }
            return contains(&argument.value, &default)
                .map(|value| Sexp::Logical(vec![Some(value)]));
        }
        let supplied: Vec<_> = args
            .iter()
            .map(|arg| CallArgument {
                name: arg.name.clone(),
                value: arg.value.clone(),
            })
            .collect();
        let matched = self.bind_supplied_argument_promises(
            &[
                FunctionParameter::required("x"),
                FunctionParameter::required("recursive"),
            ],
            &supplied,
            &self.current.clone(),
        )?;
        // These promises wrap already evaluated primitive arguments; do not evaluate their expressions again.
        let Some(Sexp::Promise { state }) = matched.get("x") else {
            return Err(EvalError::MissingArgument("x".into()));
        };
        let x = state.expression()?;
        let Some(Sexp::Promise { state }) = matched.get("recursive") else {
            return contains(&x, &default).map(|value| Sexp::Logical(vec![Some(value)]));
        };
        contains(&x, &state.expression()?).map(|value| Sexp::Logical(vec![Some(value)]))
    }
}

fn recursive_flag(value: &Sexp) -> Result<bool, EvalError> {
    if matches!(
        typeof_value(value),
        "logical" | "integer" | "double" | "character"
    ) && value_len(value) > 1
    {
        return Err(EvalError::Domain(format!(
            "'length = {}' in coercion to 'logical(1)'",
            value_len(value)
        )));
    }
    let logical = match value {
        Sexp::Factor { codes, .. } => codes.first().copied().flatten().map(|code| code != 0),
        Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_) => {
            coerce_logical_values(value)?.first().copied().flatten()
        }
        Sexp::Matrix { data, .. } => return recursive_flag(data),
        Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::List(_)
        | Sexp::Vector(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Primitive(_)
        | Sexp::InternalCall { .. }
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
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => None,
    };
    logical.ok_or_else(|| EvalError::Domain("NA in coercion to boolean".into()))
}

fn contains(value: &Sexp, recursive: &Sexp) -> Result<bool, EvalError> {
    Ok(match value {
        Sexp::Nil => false,
        Sexp::Logical(values) => values.iter().any(Option::is_none),
        Sexp::Integer(values) => values.iter().any(Option::is_none),
        Sexp::Real(values) => values.iter().any(|value| value.is_nan()),
        Sexp::String(values) => values.iter().any(Option::is_none),
        Sexp::Factor { codes, .. } => codes.iter().any(Option::is_none),
        Sexp::Matrix { data, .. } => contains(data, recursive)?,
        Sexp::DataFrame { columns, .. } => {
            let mut missing = false;
            let default = Sexp::Logical(vec![Some(false)]);
            for column in columns {
                missing |= contains(&column.value, &default)?;
            }
            missing
        }
        Sexp::List(values) => {
            let recurse = recursive_flag(recursive)?;
            for element in values {
                if if recurse {
                    contains(&element.value, recursive)?
                } else {
                    scalar_missing(&element.value)
                } {
                    return Ok(true);
                }
            }
            false
        }
        Sexp::Vector(values) => {
            let recurse = recursive_flag(recursive)?;
            for element in values {
                if if recurse {
                    contains(element, recursive)?
                } else {
                    scalar_missing(element)
                } {
                    return Ok(true);
                }
            }
            false
        }
        Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Primitive(_)
        | Sexp::InternalCall { .. }
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
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => {
            return Err(EvalError::Type(format!(
                "anyNA() applied to non-(list or vector) of type '{}'",
                typeof_value(value)
            )))
        }
    })
}

fn scalar_missing(value: &Sexp) -> bool {
    match value {
        Sexp::Logical(values) => values.as_slice() == [None],
        Sexp::Integer(values) => values.as_slice() == [None],
        Sexp::Real(values) => values.len() == 1 && values[0].is_nan(),
        Sexp::String(values) => values.as_slice() == [None],
        Sexp::Factor { codes, .. } => codes.as_slice() == [None],
        Sexp::Matrix { data, .. } => scalar_missing(data),
        Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::List(_)
        | Sexp::Vector(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Primitive(_)
        | Sexp::InternalCall { .. }
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
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => false,
    }
}
