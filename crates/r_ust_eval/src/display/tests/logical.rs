use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_logical() -> Result<(), EvalError> {
    check(
        "logical",
        Sexp::AttributedAtomic {
            values: AtomicVector::Logical(vec![Some(true), Some(false), None]),
            attributes: vec![a("note", i(&[7]))],
        },
    )
}
