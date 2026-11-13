use super::{EvalError, Sexp};

mod atomic;
mod lists;
mod runtime;
mod strings;
mod warnings;

#[cfg(test)]
mod tests;

pub(super) fn coerce_integer_values(value: &Sexp) -> Result<Vec<Option<i32>>, EvalError> {
    atomic::integers(value, strings::BlankPolicy::environment()).map(|(values, _)| values)
}

pub(super) fn silent_integer(value: &Sexp) -> Option<i32> {
    atomic::scalar_integer(value, strings::BlankPolicy::environment()).0
}
