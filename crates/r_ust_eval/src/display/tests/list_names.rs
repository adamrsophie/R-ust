use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AggregateVector;

#[test]
fn managed_print_list_names() -> Result<(), EvalError> {
    check(
        "list_names",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![i(&[1]), s(&[Some("a")])]),
            attributes: vec![
                a("names", s(&[Some("a"), None])),
                a(
                    "note",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![i(&[2])]),
                        attributes: vec![a("names", s(&[Some("x")]))],
                    },
                ),
            ],
        },
    )
}
