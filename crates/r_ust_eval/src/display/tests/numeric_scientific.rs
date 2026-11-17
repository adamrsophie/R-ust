use super::{support::check, values::a};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_numeric_scientific() -> Result<(), EvalError> {
    check(
        "numeric_scientific",
        Sexp::AttributedAtomic {
            values: AtomicVector::Real(vec![1e-9, 2.5e8, r_ust_sexp::NA_REAL]),
            attributes: vec![a("note", Sexp::Logical(vec![Some(true)]))],
        },
    )
}
