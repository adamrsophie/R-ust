use super::{
    support::check,
    values::{a, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_named_real() -> Result<(), EvalError> {
    check(
        "named_real",
        Sexp::AttributedAtomic {
            values: AtomicVector::Real(vec![1.5, 2.25, -3.0]),
            attributes: vec![a("names", s(&[Some("x"), Some("yy"), Some("z")]))],
        },
    )
}
