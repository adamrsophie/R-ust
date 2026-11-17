use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_ordered_attributes() -> Result<(), EvalError> {
    check(
        "ordered_attributes",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1)]),
            attributes: vec![
                a("z", i(&[2])),
                a("a", s(&[Some("x")])),
                a("comment", s(&[Some("hidden")])),
                a("srcref", i(&[1])),
                a("wholeSrcref", i(&[2])),
                a("srcfile", s(&[Some("hidden")])),
            ],
        },
    )
}
