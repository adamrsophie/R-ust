use super::read;
use crate::{EvalError, Sexp};
use r_ust_sexp::{
    AggregateVector, AtomicVector, Environment, ListElement, ValueImport, ValueObject,
};

#[test]
fn managed_read_preserves_atomic_metadata_and_storage() -> Result<(), EvalError> {
    let owner = Environment::global();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(7), None]),
        attributes: vec![ListElement {
            name: Some("names".into()),
            value: Sexp::String(vec![Some("first".into()), None]),
        }],
    };
    let managed = Sexp::Managed(ValueObject::import(&owner, &raw, ValueImport::Evaluated)?);
    let projected = read(&managed)?.into_owned();
    let Sexp::AttributedAtomic { values, attributes } = projected else {
        panic!("attributed atomic descriptor")
    };
    assert_eq!(values, AtomicVector::Integer(vec![Some(7), None]));
    assert_eq!(attributes[0].name.as_deref(), Some("names"));
    assert_eq!(
        attributes[0].value.snapshot()?,
        Sexp::String(vec![Some("first".into()), None])
    );
    Ok(())
}

#[test]
fn managed_read_retains_immediate_child_identity_after_origin_drop() -> Result<(), EvalError> {
    let owner = Environment::global();
    let child = ValueObject::import(
        &owner,
        &Sexp::Integer(vec![Some(9)]),
        ValueImport::Evaluated,
    )?;
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Expression(vec![child.transport()?]),
        attributes: vec![],
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Code)?;
    let projected = read(&Sexp::Managed(value.clone()))?.into_owned();
    let Sexp::AttributedAggregate {
        values: AggregateVector::Expression(values),
        ..
    } = &projected
    else {
        panic!("expression descriptor")
    };
    let Sexp::Managed(retained) = &values[0] else {
        panic!("immediate child transport")
    };
    assert!(retained.same_object(&child));
    assert_eq!(value.view()?.mode, Some(ValueImport::Code));
    drop(raw);
    drop(value);
    drop(child);
    drop(owner);
    assert_eq!(retained.snapshot()?, Sexp::Integer(vec![Some(9)]));
    Ok(())
}
