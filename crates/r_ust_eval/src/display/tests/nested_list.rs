use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AggregateVector;

#[test]
fn managed_print_nested_list() -> Result<(), EvalError> {
    check(
        "nested_list",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![Sexp::AttributedAggregate {
                values: AggregateVector::Vector(vec![i(&[1])]),
                attributes: vec![],
            }]),
            attributes: vec![
                a("names", s(&[Some("outer")])),
                a(
                    "note",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![Sexp::AttributedAggregate {
                            values: AggregateVector::Vector(vec![i(&[2])]),
                            attributes: vec![],
                        }]),
                        attributes: vec![a("names", s(&[Some("a")]))],
                    },
                ),
            ],
        },
    )
}
