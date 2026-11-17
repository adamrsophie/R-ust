use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AggregateVector;

#[test]
fn managed_print_expression_names() -> Result<(), EvalError> {
    check(
        "expression_names",
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(vec![Sexp::Symbol("a".into()), i(&[1])]),
            attributes: vec![a("names", s(&[Some("x"), None])), a("note", i(&[7]))],
        },
    )
}
