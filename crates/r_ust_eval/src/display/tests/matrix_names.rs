use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector};

#[test]
fn managed_print_matrix_names() -> Result<(), EvalError> {
    check(
        "matrix_names",
        Sexp::AttributedAtomic {
            values: AtomicVector::Real(vec![1.5, 2.25, 3.0, 4.0]),
            attributes: vec![
                a("dim", i(&[2, 2])),
                a(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![
                            s(&[Some("a"), Some("long")]),
                            s(&[Some("x"), Some("wide")]),
                        ]),
                        attributes: vec![],
                    },
                ),
                a("note", i(&[7])),
            ],
        },
    )
}
