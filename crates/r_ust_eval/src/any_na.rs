mod contains;
mod recursive;
mod scalar;
use contains::contains;
use recursive::recursive_flag;
pub(super) use scalar::scalar_missing;
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

#[cfg(test)]
mod tests;
