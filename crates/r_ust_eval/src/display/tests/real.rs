use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_real() -> Result<(), EvalError> {
    check(
        "real",
        Sexp::AttributedAtomic {
            values: AtomicVector::Real(vec![
                1.5,
                -2.25,
                r_ust_sexp::NA_REAL,
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
                -0.0,
            ]),
            attributes: vec![a("note", i(&[7]))],
        },
    )
}
