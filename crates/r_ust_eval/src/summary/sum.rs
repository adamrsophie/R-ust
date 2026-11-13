// Adapted from R 4.6.0 src/main/summary.c.
// Copyright (C) 1997-2025 The R Core Team
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka
// SPDX-License-Identifier: GPL-2.0-or-later

use super::{
    arithmetic::{add, integer_to_real},
    input::{Input, Integers},
    integer_sum::{self, Outcome},
    EvalError, EvaluatedArgument, Sexp,
};
use r_ust_sexp::NA_REAL;

struct Sum {
    integer_total: i64,
    real_total: f64,
    real_result: bool,
    use_integer_fold: bool,
    remove_missing: bool,
}

impl Sum {
    fn combine_integer(
        &mut self,
        outcome: Outcome,
        values: Integers<'_>,
    ) -> Result<Option<Sexp>, EvalError> {
        match outcome {
            Outcome::Missing => {
                return Ok(Some(if self.real_result {
                    Sexp::Real(vec![NA_REAL])
                } else {
                    Sexp::Integer(vec![None])
                }))
            }
            Outcome::Empty => {}
            Outcome::Retry => {
                self.use_integer_fold = false;
                self.real_result = true;
                // R resets from iLcum here even if earlier real arguments contributed.
                self.real_total = add(
                    integer_to_real(self.integer_total),
                    real_integer_sum(values, self.remove_missing)?.unwrap_or(0.0),
                );
            }
            Outcome::Value(total) => {
                if self.real_result {
                    self.real_total = add(self.real_total, integer_to_real(total));
                } else {
                    let sum = add(integer_to_real(self.integer_total), integer_to_real(total));
                    if sum > f64::from(i32::MAX) || sum < -f64::from(i32::MAX) {
                        self.real_result = true;
                        self.real_total = sum;
                    } else {
                        self.integer_total =
                            self.integer_total.checked_add(total).ok_or_else(|| {
                                EvalError::Domain("integer summary accumulator overflow".into())
                            })?;
                    }
                }
            }
        }
        Ok(None)
    }

    fn integer(&mut self, values: Integers<'_>) -> Result<Option<Sexp>, EvalError> {
        if self.use_integer_fold {
            let outcome = integer_sum::fold(values.clone(), self.remove_missing)?;
            self.combine_integer(outcome, values)
        } else {
            if let Some(value) = real_integer_sum(values, self.remove_missing)? {
                self.real_total = add(self.real_total, value);
            }
            Ok(None)
        }
    }

    fn result(self) -> Result<Sexp, EvalError> {
        if self.real_result {
            Ok(Sexp::Real(vec![self.real_total]))
        } else {
            Ok(Sexp::Integer(vec![Some(
                i32::try_from(self.integer_total).map_err(|_| {
                    EvalError::Domain("integer summary result exceeds the integer range".into())
                })?,
            )]))
        }
    }
}

fn real_integer_sum(values: Integers<'_>, remove_missing: bool) -> Result<Option<f64>, EvalError> {
    let mut total = 0.0;
    let mut updated = false;
    for value in values {
        match value? {
            Some(value) => {
                total = add(total, f64::from(value));
                updated = true;
            }
            None if !remove_missing => return Ok(Some(NA_REAL)),
            None => {}
        }
    }
    Ok(updated.then_some(total))
}

pub(super) fn evaluate(
    args: &[EvaluatedArgument],
    remove_missing: bool,
) -> Result<Sexp, EvalError> {
    let inputs = args
        .iter()
        .filter(|arg| arg.name.as_deref() != Some("na.rm"))
        .map(|arg| Input::parse(&arg.value))
        .collect::<Result<Vec<_>, _>>()?;
    let mut sum = Sum {
        integer_total: 0,
        real_total: 0.0,
        real_result: inputs.iter().any(|input| matches!(input, Input::Real(_))),
        use_integer_fold: true,
        remove_missing,
    };
    for input in inputs {
        match input {
            Input::Integer(values) => {
                if let Some(answer) = sum.integer(values)? {
                    return Ok(answer);
                }
            }
            Input::Real(values) => {
                let mut total = 0.0;
                let mut updated = false;
                for value in values {
                    if !remove_missing || !value.is_nan() {
                        total = add(total, *value);
                        updated = true;
                    }
                }
                if updated {
                    sum.real_total = add(sum.real_total, total);
                }
            }
        }
    }
    sum.result()
}

#[cfg(test)]
mod tests {
    use super::{Integers, Outcome, Sum};

    #[test]
    fn retry_resums_the_argument_resets_from_integer_total_and_stays_in_real_mode() {
        for real_result in [false, true] {
            let mut sum = Sum {
                integer_total: 7,
                real_total: 100.0,
                real_result,
                use_integer_fold: true,
                remove_missing: false,
            };
            let values = [Some(2), Some(3)];
            assert_eq!(
                sum.combine_integer(Outcome::Retry, Integers::Integer(values.iter()))
                    .unwrap(),
                None
            );
            assert!(sum.real_result);
            assert!(!sum.use_integer_fold);
            assert_eq!(sum.real_total.to_bits(), 12.0_f64.to_bits());
            let values = [Some(i32::MAX), Some(-i32::MAX)];
            assert_eq!(sum.integer(Integers::Integer(values.iter())).unwrap(), None);
            assert_eq!(sum.real_total.to_bits(), 12.0_f64.to_bits());
        }
    }

    #[test]
    fn integer_missing_after_fallback_uses_outer_nan_arithmetic() {
        let mut sum = Sum {
            integer_total: 0,
            real_total: f64::NAN,
            real_result: true,
            use_integer_fold: false,
            remove_missing: false,
        };
        assert_eq!(sum.integer(Integers::Integer([None].iter())).unwrap(), None);
        assert_eq!(sum.real_total.to_bits(), 0x7ff8_0000_0000_07a2);
    }
}
