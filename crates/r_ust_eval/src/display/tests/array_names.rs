use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector};

#[test]
fn managed_print_array_names() -> Result<(), EvalError> {
    check(
        "array_names",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer((1..=8).map(Some).collect()),
            attributes: vec![
                a("dim", i(&[2, 2, 2])),
                a(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![
                            s(&[Some("a"), Some("b")]),
                            s(&[Some("x"), Some("y")]),
                            s(&[Some("one"), Some("two")]),
                        ]),
                        attributes: vec![],
                    },
                ),
            ],
        },
    )
}
