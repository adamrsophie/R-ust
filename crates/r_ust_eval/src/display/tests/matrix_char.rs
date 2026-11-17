use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector};

#[test]
fn managed_print_matrix_char() -> Result<(), EvalError> {
    check(
        "matrix_char",
        Sexp::AttributedAtomic {
            values: AtomicVector::String(vec![
                Some("a".into()),
                Some("bb".into()),
                None,
                Some("z".into()),
            ]),
            attributes: vec![
                a("dim", i(&[2, 2])),
                a(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![
                            s(&[Some("a"), Some("b")]),
                            s(&[Some("x"), Some("y")]),
                        ]),
                        attributes: vec![],
                    },
                ),
            ],
        },
    )
}
