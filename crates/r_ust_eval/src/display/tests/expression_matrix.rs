use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AggregateVector;

#[test]
fn managed_print_expression_matrix() -> Result<(), EvalError> {
    check(
        "expression_matrix",
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(vec![
                Sexp::Symbol("a".into()),
                Sexp::Symbol("b".into()),
                Sexp::Symbol("c".into()),
                Sexp::Symbol("d".into()),
            ]),
            attributes: vec![a("dim", i(&[2, 2]))],
        },
    )
}
