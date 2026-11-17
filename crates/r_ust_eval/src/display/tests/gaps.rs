use super::values::{a, i, s};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector, Environment, ValueImport, ValueObject};

fn compare(name: &str, raw: Sexp) -> Result<(), EvalError> {
    let expected = include_str!("gap_reference.txt")
        .split(&format!("CASE {name}\n"))
        .nth(1)
        .expect("reference case")
        .split("\nEND\n")
        .next()
        .expect("reference end");
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    for value in [&raw, &object.transport()?] {
        assert_eq!(super::super::format_sexp(value)?, expected, "{name}");
    }
    Ok(())
}

#[test]
fn managed_print_gap_list_array_factor_format_dispatch() -> Result<(), EvalError> {
    let factor = |values| Sexp::AttributedAtomic {
        values: AtomicVector::Integer(values),
        attributes: vec![
            a("levels", s(&[Some("a"), Some("b")])),
            a("class", s(&[Some("factor")])),
        ],
    };
    compare(
        "list_factor_matrix",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![
                factor(vec![Some(1)]),
                factor(vec![Some(1), Some(2)]),
            ]),
            attributes: vec![a("dim", i(&[1, 2]))],
        },
    )
}

#[test]
fn managed_print_list_array_character_truncation() -> Result<(), EvalError> {
    compare(
        "list_long_character",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![Sexp::String(vec![Some("x".repeat(101))])]),
            attributes: vec![a("dim", i(&[1, 1]))],
        },
    )
}
