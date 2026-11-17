use super::{
    support::check,
    values::{a, i},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AggregateVector;

#[test]
fn managed_print_expression() -> Result<(), EvalError> {
    check(
        "expression",
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(vec![
                Sexp::Symbol("a".into()),
                i(&[1]),
                Sexp::Call {
                    function: Box::new(Sexp::Symbol("+".into())),
                    arguments: vec![
                        crate::CallArgument::positional(Sexp::Symbol("x".into())),
                        crate::CallArgument::positional(Sexp::Real(vec![2.0])),
                    ],
                },
            ]),
            attributes: vec![a("note", i(&[7]))],
        },
    )
}
