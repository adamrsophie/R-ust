use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_character() -> Result<(), EvalError> {
    check(
        "character",
        Sexp::AttributedAtomic {
            values: AtomicVector::String(vec![Some("a".into()), Some("long".into()), None]),
            attributes: vec![a("note", i(&[7]))],
        },
    )
}
