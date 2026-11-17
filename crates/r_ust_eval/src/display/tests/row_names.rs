use super::{
    support::check,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::AggregateVector;

#[test]
fn managed_print_row_names() -> Result<(), EvalError> {
    check(
        "row_names",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![i(&[1, 2, 3])]),
            attributes: vec![
                a("names", s(&[Some("a")])),
                a("class", s(&[Some("data.frame")])),
                a("row.names", Sexp::Integer(vec![None, Some(-3)])),
                a("note", i(&[7])),
            ],
        },
    )
}
