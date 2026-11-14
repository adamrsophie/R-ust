mod recursive;
use recursive::recursive_flag;
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
    pub(super) fn eval_any_na(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let mut values = Vec::new();
        for (index, argument) in arguments.iter().enumerate() {
            if matches!(argument.value, Sexp::MissingArg(_)) {
                return Err(crate::SourceFault::arity(
                    format!("argument {} is empty", index + 1),
                    call,
                ));
            }
            values.extend(self.eval_arguments(
                std::slice::from_ref(argument),
                false,
                Some(call),
            )?);
        }
        self.visible = true;
        self.call_any_na(&values, Some(call))
    }

    pub(super) fn call_any_na(
        &mut self,
        args: &[EvaluatedArgument],
        call: Option<&Sexp>,
    ) -> Result<Sexp, EvalError> {
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
        let matched = self.bind_promised_arguments(
            &[
                FunctionParameter::required("x"),
                FunctionParameter::required("recursive"),
            ],
            &supplied,
            call,
        )?;
        let Some(x) = matched.get("x") else {
            return Err(EvalError::MissingArgument("x".into()));
        };
        let recursive = matched.get("recursive").unwrap_or(&default);
        contains(x, recursive).map(|value| Sexp::Logical(vec![Some(value)]))
    }
}

fn contains(value: &Sexp, recursive: &Sexp) -> Result<bool, EvalError> {
    if let Sexp::PairList(values) = value {
        return contains(&Sexp::List(values.as_slice().to_vec()), recursive);
    }
    if matches!(value, Sexp::ListObject(_) | Sexp::PairListObject(_)) {
        if recursive_flag(recursive)? {
            crate::ordinary_cycles::check(value, "cyclic ordinary value anyNA")?;
        }
        return contains(crate::ordinary_values::read(value)?.as_ref(), recursive);
    }
    Ok(match value {
        Sexp::Nil => false,
        Sexp::Logical(values) => values.iter().any(Option::is_none),
        Sexp::Integer(values) => values.iter().any(Option::is_none),
        Sexp::Real(values) => values.iter().any(|value| value.is_nan()),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            values.iter().any(Option::is_none)
        }
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
        Sexp::ClassedList {
            elements: values, ..
        }
        | Sexp::List(values) => {
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
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Char(_)
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Environment(_)
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
        | Sexp::ExtractArguments { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::MemberAssignment { .. }
        | Sexp::ReplacementAssignment { .. }
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

pub(super) fn scalar_missing(value: &Sexp) -> bool {
    match value {
        Sexp::Logical(values) => values.as_slice() == [None],
        Sexp::Integer(values) => values.as_slice() == [None],
        Sexp::Real(values) => values.len() == 1 && values[0].is_nan(),
        Sexp::String(values) | Sexp::AttributedCharacter { values, .. } => {
            values.as_slice() == [None]
        }
        Sexp::Factor { codes, .. } => codes.as_slice() == [None],
        Sexp::Matrix { data, .. } => scalar_missing(data),
        Sexp::Nil
        | Sexp::MissingArg(_)
        | Sexp::Symbol(_)
        | Sexp::MissingValue
        | Sexp::PairList(_)
        | Sexp::ListObject(_)
        | Sexp::PairListObject(_)
        | Sexp::Char(_)
        | Sexp::ClassedList { .. }
        | Sexp::List(_)
        | Sexp::Vector(_)
        | Sexp::DataFrame { .. }
        | Sexp::Dots(_)
        | Sexp::Promise { .. }
        | Sexp::Closure { .. }
        | Sexp::Environment(_)
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
        | Sexp::ExtractArguments { .. }
        | Sexp::Extract { .. }
        | Sexp::Dollar { .. }
        | Sexp::Call { .. }
        | Sexp::Assignment { .. }
        | Sexp::SuperAssignment { .. }
        | Sexp::Subassignment { .. }
        | Sexp::MatrixSubassignment { .. }
        | Sexp::ExtractAssignment { .. }
        | Sexp::DollarAssignment { .. }
        | Sexp::MemberAssignment { .. }
        | Sexp::ReplacementAssignment { .. }
        | Sexp::NamesAssignment { .. }
        | Sexp::Binary { .. }
        | Sexp::Unary { .. }
        | Sexp::Parenthesized(_) => false,
    }
}
