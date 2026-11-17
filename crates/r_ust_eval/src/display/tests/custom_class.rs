use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_custom_class() -> Result<(), EvalError> {
    check(
        "custom_class",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer((1..=3).map(Some).collect()),
            attributes: vec![a("class", s(&[Some("custom")])), a("note", i(&[7]))],
        },
    )
}
