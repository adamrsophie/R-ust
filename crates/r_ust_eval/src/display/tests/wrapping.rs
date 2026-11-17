use super::{support::check, values::a};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_wrapping() -> Result<(), EvalError> {
    check(
        "wrapping",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer((1..=35).map(Some).collect()),
            attributes: vec![a("note", Sexp::Logical(vec![Some(true)]))],
        },
    )
}
