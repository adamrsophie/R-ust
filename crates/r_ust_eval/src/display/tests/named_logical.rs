use super::{
    support::check,
    values::{a, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_named_logical() -> Result<(), EvalError> {
    check(
        "named_logical",
        Sexp::AttributedAtomic {
            values: AtomicVector::Logical(vec![Some(true), Some(false), None]),
            attributes: vec![a("names", s(&[Some("a"), Some(""), None]))],
        },
    )
}
