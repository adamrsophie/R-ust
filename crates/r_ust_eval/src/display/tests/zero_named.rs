use super::{
    support::check,
    values::{a, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_zero_named() -> Result<(), EvalError> {
    check(
        "zero_named",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![]),
            attributes: vec![a("names", s(&[]))],
        },
    )
}
