use super::{
    support::layout,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AggregateVector;

#[test]
fn managed_print_list_matrix_summaries() -> Result<(), EvalError> {
    for (name, values) in [
        (
            "list_matrix",
            vec![i(&[1]), i(&[1, 2]), s(&[Some("x")]), Sexp::Nil],
        ),
        (
            "list_matrix_character_na",
            vec![
                s(&[None]),
                Sexp::Logical(vec![Some(true)]),
                Sexp::Real(vec![r_ust_sexp::NA_REAL]),
                Sexp::AttributedAggregate {
                    values: AggregateVector::Vector(vec![i(&[1])]),
                    attributes: vec![],
                },
            ],
        ),
    ] {
        layout(
            name,
            Sexp::AttributedAggregate {
                values: AggregateVector::Vector(values),
                attributes: vec![a("dim", i(&[2, 2]))],
            },
        )?;
    }
    Ok(())
}

#[test]
fn managed_print_one_dim_list_names() -> Result<(), EvalError> {
    layout(
        "list_one_dim",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![i(&[1]), i(&[2])]),
            attributes: vec![
                a("dim", i(&[2])),
                a(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![s(&[Some("a"), Some("b")])]),
                        attributes: vec![],
                    },
                ),
            ],
        },
    )
}
