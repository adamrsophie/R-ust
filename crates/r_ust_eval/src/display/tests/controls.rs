use super::{
    support::{deparse, expected},
    values::{a, s},
};
use crate::{EvalError, Sexp};
use r_ust_sexp::{AtomicVector, Environment, ValueImport, ValueObject};

#[test]
fn managed_print_deparse_controls_preserve_metadata() -> Result<(), EvalError> {
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1), Some(20), None]),
        attributes: vec![
            a("names", s(&[Some("a"), Some("long"), None])),
            a("note", Sexp::Integer(vec![Some(7)])),
        ],
    };
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    for value in [&raw, &object.transport()?] {
        for (control, name) in [
            (0, "none"),
            (4, "attributes"),
            (1024, "nice"),
            (1631, "all"),
        ] {
            assert_eq!(
                deparse(value, control, false)?,
                expected(&format!("control_{name}")),
                "{name}"
            );
        }
    }
    Ok(())
}

#[test]
fn managed_print_unquoted_character_names_and_attributes() -> Result<(), EvalError> {
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::String(vec![Some("a".into()), Some("long".into()), None]),
        attributes: vec![a("names", s(&[Some("x"), Some("yy"), None]))],
    };
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    for value in [&raw, &object.transport()?] {
        assert_eq!(
            super::super::character::format(value, false)?,
            expected("unquoted_named_character")
        );
    }
    Ok(())
}
