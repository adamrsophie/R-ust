use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AggregateVector;

#[test]
fn managed_print_list() -> Result<(), EvalError> {
    check(
        "list",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![i(&[1]), s(&[Some("a")])]),
            attributes: vec![a("note", i(&[7]))],
        },
    )
}
