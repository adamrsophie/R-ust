use super::{
    support::layout,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector};

#[test]
fn managed_print_dimension_titles() -> Result<(), EvalError> {
    for (name, dim, count) in [
        ("matrix_titles", vec![2, 2], 4),
        ("array_titles", vec![2, 2, 2], 8),
    ] {
        let mut labels = vec![s(&[Some("a"), Some("b")]), s(&[Some("x"), Some("y")])];
        let mut titles = vec![Some("rows"), Some("cols")];
        if dim.len() == 3 {
            labels.push(s(&[Some("one"), Some("two")]));
            titles.push(Some("layers"));
        }
        layout(
            name,
            Sexp::AttributedAtomic {
                values: AtomicVector::Integer((1..=count).map(Some).collect()),
                attributes: vec![
                    a("dim", i(&dim)),
                    a(
                        "dimnames",
                        Sexp::AttributedAggregate {
                            values: AggregateVector::Vector(labels),
                            attributes: vec![a("names", s(&titles))],
                        },
                    ),
                ],
            },
        )?;
    }
    Ok(())
}

#[test]
fn managed_print_one_dim_not_sequence() -> Result<(), EvalError> {
    layout(
        "one_dim_not_sequence",
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(3)]),
            attributes: vec![
                a("dim", i(&[2])),
                a(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![s(&[Some("a"), Some("b")])]),
                        attributes: vec![a("names", s(&[Some("axis")]))],
                    },
                ),
            ],
        },
    )
}
