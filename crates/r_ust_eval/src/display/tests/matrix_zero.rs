use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_matrix_zero() -> Result<(), EvalError> {
    check(
        "matrix_zero",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![]),
            attributes: vec![a("dim", i(&[0, 3]))],
        },
    )
}
