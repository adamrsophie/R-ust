use super::Comparison;
use crate::{EvalError, Sexp};
use r_ust_sexp::{
    AggregateVector, AtomicVector, Environment, ListElement, ValueImport, ValueObject, NA_REAL,
};

#[test]
fn typed_and_managed_atomic_identity_uses_real_kernel_and_metadata() -> Result<(), EvalError> {
    let owner = Environment::global();
    let plain = Sexp::Real(vec![NA_REAL, f64::NAN, -0.0]);
    let typed = Sexp::AttributedAtomic {
        values: AtomicVector::Real(vec![NA_REAL, f64::from_bits(0x7ff8_0000_0000_0001), 0.0]),
        attributes: vec![],
    };
    assert!(Comparison::default().compare(&plain, &typed)?);
    let managed = Sexp::Managed(ValueObject::import(&owner, &typed, ValueImport::Evaluated)?);
    assert!(Comparison::default().compare(&plain, &managed)?);
    let wrong_na = Sexp::Real(vec![f64::NAN, f64::NAN, 0.0]);
    assert!(!Comparison::default().compare(&wrong_na, &managed)?);
    Ok(())
}

#[test]
fn managed_object_flag_tracks_class_removal_without_losing_payload() -> Result<(), EvalError> {
    let owner = Environment::global();
    let value = ValueObject::import(
        &owner,
        &Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(7)]),
            attributes: vec![ListElement {
                name: Some("class".into()),
                value: Sexp::String(vec![Some("custom".into())]),
            }],
        },
        ValueImport::Evaluated,
    )?;
    let unclassed = value.without_class()?;
    assert!(super::typed::object_flag(&Sexp::Managed(value.clone()))?);
    assert!(!super::typed::object_flag(&Sexp::Managed(
        unclassed.clone()
    ))?);
    assert!(!Comparison::default().compare(&Sexp::Managed(value), &Sexp::Managed(unclassed))?);
    Ok(())
}

#[test]
fn typed_attributes_obey_set_order_and_expand_compact_rows_in_both_modes() -> Result<(), EvalError>
{
    let attribute = |name: &str, value| ListElement {
        name: Some(name.into()),
        value,
    };
    let make = |attributes| Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![]),
        attributes,
    };
    let compact = make(vec![
        attribute("row.names", Sexp::Integer(vec![None, Some(-3)])),
        attribute("note", Sexp::Integer(vec![Some(7)])),
    ]);
    let expanded = make(vec![
        attribute("row.names", Sexp::Integer(vec![Some(1), Some(2), Some(3)])),
        attribute("note", Sexp::Integer(vec![Some(7)])),
    ]);
    for ordered in [false, true] {
        let mut comparison = Comparison::default();
        comparison.ordered_attributes = ordered;
        assert!(comparison.compare(&compact, &expanded)?);
    }
    let reversed = make(vec![
        attribute("note", Sexp::Integer(vec![Some(7)])),
        attribute("row.names", Sexp::Integer(vec![Some(1), Some(2), Some(3)])),
    ]);
    assert!(Comparison::default().compare(&compact, &reversed)?);
    let mut comparison = Comparison::default();
    comparison.ordered_attributes = true;
    assert!(!comparison.compare(&compact, &reversed)?);
    Ok(())
}
