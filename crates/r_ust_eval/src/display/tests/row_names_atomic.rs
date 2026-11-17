use super::{support::check, values::a};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_row_names_atomic() -> Result<(), EvalError> {
    check(
        "row_names_atomic",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer((1..=3).map(Some).collect()),
            attributes: vec![a("row.names", Sexp::Integer(vec![None, Some(-3)]))],
        },
    )
}
