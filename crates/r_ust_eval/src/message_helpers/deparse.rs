use crate::{EvalError, Sexp};

pub(super) fn deparse_value(value: &Sexp) -> Result<String, EvalError> {
    crate::language_print::format_coercion(value)
}
