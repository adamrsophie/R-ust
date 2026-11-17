use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_zero_character() -> Result<(), EvalError> {
    check(
        "zero_character",
        Sexp::AttributedAtomic {
            values: AtomicVector::String(vec![]),
            attributes: vec![a("note", i(&[7]))],
        },
    )
}
