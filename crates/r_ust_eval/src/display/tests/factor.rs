use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_factor() -> Result<(), EvalError> {
    check(
        "factor",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(2), None]),
            attributes: vec![
                a("levels", s(&[Some("a"), Some("b")])),
                a("class", s(&[Some("factor")])),
                a("note", i(&[7])),
            ],
        },
    )
}
