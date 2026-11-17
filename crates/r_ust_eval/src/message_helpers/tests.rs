use super::list_character;
use crate::{EvalError, Sexp};
use r_ust_sexp::{AtomicVector, Environment, ListElement, ValueImport, ValueObject};

#[test]
fn list_character_uses_nice_names_but_keeps_scalar_string_exception() -> Result<(), EvalError> {
    let owner = Environment::global();
    for (values, expected) in [
        (AtomicVector::Integer(vec![Some(7)]), "c(label = 7)"),
        (AtomicVector::String(vec![Some("kept".into())]), "kept"),
    ] {
        let raw = Sexp::AttributedAtomic {
            values,
            attributes: vec![ListElement {
                name: Some("names".into()),
                value: Sexp::String(vec![Some("label".into())]),
            }],
        };
        assert_eq!(list_character(&raw)?, expected);
        let value = Sexp::Managed(ValueObject::import(&owner, &raw, ValueImport::Evaluated)?);
        assert_eq!(list_character(&value)?, expected);
    }
    Ok(())
}
