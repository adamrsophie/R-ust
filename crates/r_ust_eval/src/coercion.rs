use super::{EvalError, Sexp};

mod atomic;
mod character;
mod lists;
pub(super) mod logical;
mod runtime;
pub(crate) mod storage;
mod strings;
mod vector;
mod warnings;

#[cfg(test)]
mod tests;

pub(super) fn coerce_integer_values(value: &Sexp) -> Result<Vec<Option<i32>>, EvalError> {
    atomic::integers(value, strings::BlankPolicy::environment()).map(|(values, _)| values)
}

pub(super) fn silent_integer(value: &Sexp) -> Option<i32> {
    atomic::scalar_integer(value, strings::BlankPolicy::environment()).0
}

#[cfg(test)]
mod managed_tests;
