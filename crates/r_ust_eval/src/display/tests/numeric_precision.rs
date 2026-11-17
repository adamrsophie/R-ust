use super::{support::check, values::a};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_numeric_precision() -> Result<(), EvalError> {
    check(
        "numeric_precision",
        Sexp::AttributedAtomic {
            values: AtomicVector::Real(vec![1.0, 1.23456789, 1000000.0]),
            attributes: vec![a("note", Sexp::Logical(vec![Some(true)]))],
        },
    )
}
