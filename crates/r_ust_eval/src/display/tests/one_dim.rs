use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector};

#[test]
fn managed_print_one_dim() -> Result<(), EvalError> {
    check(
        "one_dim",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer((1..=3).map(Some).collect()),
            attributes: vec![
                a("dim", i(&[3])),
                a(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![s(&[
                            Some("a"),
                            Some("b"),
                            Some("c"),
                        ])]),
                        attributes: vec![],
                    },
                ),
                a("note", i(&[7])),
            ],
        },
    )
}
