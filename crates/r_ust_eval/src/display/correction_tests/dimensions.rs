use super::{attribute, check, expected};
use crate::language_print::{format_coercion, format_deparse, Profile};
use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::{AggregateVector, AtomicVector, Environment, ValueImport, ValueObject};

#[test]
fn managed_print_mp1_null_and_character_dimnames_match_raw_and_retained_views(
) -> Result<(), EvalError> {
    for (label, names) in [
        ("null", Sexp::Nil),
        (
            "character",
            Sexp::String(vec![Some("a".into()), Some("b".into()), Some("c".into())]),
        ),
    ] {
        let raw = Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(2), Some(3)]),
            attributes: vec![
                attribute("dim", Sexp::Integer(vec![Some(3)])),
                attribute(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![names]),
                        attributes: vec![],
                    },
                ),
            ],
        };
        check(&format!("{label}_dim_print"), raw.clone())?;
        let owner = Environment::default();
        let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        let descriptor = object.borrow()?;
        let transport = object.transport()?;
        let before = object.view()?;
        drop(owner);
        drop(object);
        for value in [&raw, &transport, descriptor.value()] {
            let (lines, incomplete) = format_deparse(
                &mut RuntimeState::default(),
                value,
                Profile {
                    cutoff: 60,
                    backtick: false,
                    control: 1093,
                    lines: None,
                    utf8: true,
                },
            )?;
            assert!(!incomplete);
            assert_eq!(
                lines.join("\n"),
                expected(&format!("{label}_dim_deparse")),
                "{label}"
            );
            assert_eq!(format_coercion(value)?, "1:3");
        }
        let after = descriptor.owner().view()?;
        for (a, b) in before.children.iter().zip(&after.children) {
            assert!(a.same_object(b));
        }
    }
    Ok(())
}
