use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_array() -> Result<(), EvalError> {
    check(
        "array",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer((1..=8).map(Some).collect()),
            attributes: vec![a("dim", i(&[2, 2, 2]))],
        },
    )
}
