use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_named_integer() -> Result<(), EvalError> {
    check(
        "named_integer",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(20), None]),
            attributes: vec![
                a("names", s(&[Some("a"), Some("long"), None])),
                a("note", i(&[7])),
            ],
        },
    )
}
