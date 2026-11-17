use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_zero_integer() -> Result<(), EvalError> {
    check(
        "zero_integer",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![]),
            attributes: vec![a("note", i(&[7]))],
        },
    )
}
