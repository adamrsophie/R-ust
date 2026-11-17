use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_matrix() -> Result<(), EvalError> {
    check(
        "matrix",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer((1..=6).map(Some).collect()),
            attributes: vec![a("dim", i(&[2, 3])), a("note", i(&[7]))],
        },
    )
}
