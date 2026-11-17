use super::{
    support::check,
    values::{a, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_empty_names() -> Result<(), EvalError> {
    check(
        "empty_names",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(2)]),
            attributes: vec![a("names", s(&[Some(""), Some("")]))],
        },
    )
}
