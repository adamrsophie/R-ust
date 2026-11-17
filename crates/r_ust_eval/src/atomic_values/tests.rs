use super::*;
use r_ust_sexp::{AtomicVector, Environment, ValueImport, ValueObject};

#[test]
fn typed_atomic_consumers_read_payload_and_leave_metadata_intact() -> Result<(), EvalError> {
    let owner = Environment::global();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(7), None]),
        attributes: vec![ListElement {
            name: Some("note".into()),
            value: Sexp::Symbol("never_evaluate".into()),
        }],
    };
    let managed = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let before = managed.snapshot()?;
    for value in [&raw, &Sexp::Managed(managed.clone())] {
        let numbers = numeric_values(value)?;
        assert_eq!(numbers[0], 7.0);
        assert!(is_na_real(numbers[1]));
        assert_eq!(logical_values(value)?, vec![Some(true), None]);
        assert_eq!(string_values(value)?, vec![Some("7".into()), None]);
    }
    assert_eq!(managed.snapshot()?, before);
    Ok(())
}

#[test]
fn typed_character_flags_and_scalar_options_use_original_coercion_rules() -> Result<(), EvalError> {
    let owner = Environment::global();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::String(vec![Some("TRUE".into()), Some("invalid".into())]),
        attributes: vec![],
    };
    let managed = Sexp::Managed(ValueObject::import(&owner, &raw, ValueImport::Evaluated)?);
    for value in [&raw, &managed] {
        assert_eq!(coerce_logical_values(value)?, vec![Some(true), None]);
        assert_eq!(string_scalar(value, "option")?, "TRUE");
        assert!(numeric_values(value).is_err());
    }
    Ok(())
}
