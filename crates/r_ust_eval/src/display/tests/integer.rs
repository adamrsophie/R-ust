use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_integer() -> Result<(), EvalError> {
    check(
        "integer",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(-2), None]),
            attributes: vec![a("note", i(&[7]))],
        },
    )
}
