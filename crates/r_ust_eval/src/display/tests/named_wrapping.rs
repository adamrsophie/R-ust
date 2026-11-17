use super::{support::check, values::a};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_named_wrapping() -> Result<(), EvalError> {
    check(
        "named_wrapping",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer((1..=18).map(Some).collect()),
            attributes: vec![a("names", Sexp::String(vec![Some("longname".into()); 18]))],
        },
    )
}
