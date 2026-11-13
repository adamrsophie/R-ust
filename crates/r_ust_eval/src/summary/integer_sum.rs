// Adapted from R 4.6.0 src/main/summary.c.
// Copyright (C) 1997-2025 The R Core Team
// Copyright (C) 1995, 1996 Robert Gentleman and Ross Ihaka
// SPDX-License-Identifier: GPL-2.0-or-later

use super::{input::Integers, EvalError};

#[derive(Debug, PartialEq)]
pub(super) enum Outcome {
    Empty,
    Value(i64),
    Missing,
    Retry,
}

struct ExactSum {
    total: i64,
    counter: i32,
    updated: bool,
}

impl ExactSum {
    fn new() -> Self {
        Self {
            total: 0,
            counter: -i32::MAX,
            updated: false,
        }
    }

    fn add(
        &mut self,
        value: Option<i32>,
        remove_missing: bool,
    ) -> Result<Option<Outcome>, EvalError> {
        let Some(value) = value else {
            return Ok(if remove_missing {
                None
            } else {
                Some(Outcome::Missing)
            });
        };
        self.updated = true;
        self.total = self
            .total
            .checked_add(i64::from(value))
            .ok_or_else(|| EvalError::Domain("integer summary accumulator overflow".into()))?;
        let previous = self.counter;
        self.counter += 1;
        if previous > 1000 {
            if self.total > 9_000_000_000_000_000 || self.total < -9_000_000_000_000_000 {
                return Ok(Some(Outcome::Retry));
            }
            self.counter = 0;
        }
        Ok(None)
    }
}

pub(super) fn fold(values: Integers<'_>, remove_missing: bool) -> Result<Outcome, EvalError> {
    let mut sum = ExactSum::new();
    for value in values {
        if let Some(outcome) = sum.add(value?, remove_missing)? {
            return Ok(outcome);
        }
    }
    Ok(if sum.updated {
        Outcome::Value(sum.total)
    } else {
        Outcome::Empty
    })
}

#[cfg(test)]
mod tests {
    use super::{ExactSum, Outcome};

    #[test]
    fn overflow_counter_advances_only_for_nonmissing_values_and_checks_after_increment() {
        let mut sum = ExactSum::new();
        assert_eq!(sum.counter, -i32::MAX);
        assert_eq!(sum.add(None, true).unwrap(), None);
        assert_eq!(sum.counter, -i32::MAX);
        assert_eq!(sum.add(Some(1), true).unwrap(), None);
        assert_eq!(sum.counter, -i32::MAX + 1);
        sum.counter = 1000;
        assert_eq!(sum.add(Some(0), false).unwrap(), None);
        assert_eq!(sum.counter, 1001);
        assert_eq!(sum.add(None, false).unwrap(), Some(Outcome::Missing));
        assert_eq!(sum.counter, 1001);
        assert_eq!(sum.add(Some(0), false).unwrap(), None);
        assert_eq!(sum.counter, 0);
    }

    #[test]
    fn overflow_threshold_is_strict_and_signed_and_requests_argument_resumming() {
        for sign in [-1, 1] {
            let mut sum = ExactSum {
                total: sign * 9_000_000_000_000_000,
                counter: 1001,
                updated: true,
            };
            assert_eq!(sum.add(Some(0), false).unwrap(), None);
            assert_eq!(sum.counter, 0);
            sum.counter = 1001;
            assert_eq!(
                sum.add(Some(i32::try_from(sign).unwrap()), false).unwrap(),
                Some(Outcome::Retry)
            );
            assert_eq!(sum.counter, 1002);
        }
    }
}
