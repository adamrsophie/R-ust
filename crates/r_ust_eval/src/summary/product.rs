// Adapted from R 4.6.0 src/main/summary.c.
// Copyright (C) 1997-2025 The R Core Team
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka
// SPDX-License-Identifier: GPL-2.0-or-later

use super::{
    arithmetic::multiply,
    input::{Input, Integers},
    EvalError, EvaluatedArgument, Sexp,
};
use r_ust_sexp::NA_REAL;

fn integer_product(values: Integers<'_>, remove_missing: bool) -> Result<Option<f64>, EvalError> {
    let mut total = 1.0;
    let mut updated = false;
    for value in values {
        match value? {
            Some(value) => {
                total = multiply(total, f64::from(value));
                updated = true;
            }
            None if !remove_missing => return Ok(Some(NA_REAL)),
            None => {}
        }
        if total.is_nan() {
            return Ok(Some(NA_REAL));
        }
    }
    Ok(updated.then_some(total))
}

pub(super) fn evaluate(
    args: &[EvaluatedArgument],
    remove_missing: bool,
) -> Result<Sexp, EvalError> {
    let mut product = 1.0;
    for arg in args
        .iter()
        .filter(|arg| arg.name.as_deref() != Some("na.rm"))
    {
        let subtotal = match Input::parse(&arg.value)? {
            Input::Integer(values) => integer_product(values, remove_missing)?,
            Input::Real(values) => {
                let mut total = 1.0;
                let mut updated = false;
                for value in values {
                    if !remove_missing || !value.is_nan() {
                        total = multiply(total, *value);
                        updated = true;
                    }
                }
                updated.then_some(total)
            }
        };
        if let Some(value) = subtotal {
            product = multiply(product, value);
        }
    }
    Ok(Sexp::Real(vec![product]))
}
