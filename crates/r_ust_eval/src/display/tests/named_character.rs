use super::{
    support::check,
    values::{a, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AtomicVector;

#[test]
fn managed_print_named_character() -> Result<(), EvalError> {
    check(
        "named_character",
        Sexp::AttributedAtomic {
            values: AtomicVector::String(vec![Some("a".into()), Some("long".into()), None]),
            attributes: vec![a("names", s(&[Some("x"), Some("yy"), None]))],
        },
    )
}
