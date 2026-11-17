use super::{
    support::expected,
    values::{a, i, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector, Environment, ValueImport, ValueObject};

#[test]
fn managed_print_unquoted_plain_and_matrix() -> Result<(), EvalError> {
    let owner = Environment::default();
    for (name, raw) in [
        (
            "unquoted_character",
            Sexp::AttributedAtomic {
                values: AtomicVector::String(vec![Some("a".into()), Some("long".into()), None]),
                attributes: vec![a("note", i(&[7]))],
            },
        ),
        (
            "unquoted_matrix_char",
            Sexp::AttributedAtomic {
                values: AtomicVector::String(vec![
                    Some("a".into()),
                    Some("bb".into()),
                    None,
                    Some("z".into()),
                ]),
                attributes: vec![
                    a("dim", i(&[2, 2])),
                    a(
                        "dimnames",
                        Sexp::AttributedAggregate {
                            values: AggregateVector::Vector(vec![
                                s(&[Some("a"), Some("b")]),
                                s(&[Some("x"), Some("y")]),
                            ]),
                            attributes: vec![],
                        },
                    ),
                ],
            },
        ),
    ] {
        let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        for value in [&raw, &object.transport()?] {
            assert_eq!(
                super::super::character::format(value, false)?,
                expected(name),
                "{name}"
            );
        }
    }
    Ok(())
}
