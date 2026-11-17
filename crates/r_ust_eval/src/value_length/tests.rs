use super::value_len;
use crate::{EvalError, Sexp};
use r_ust_sexp::{
    AggregateVector, AtomicVector, Environment, EvaluationOrigin, Force, ListElement, PromiseState,
    ValueImport, ValueObject,
};

#[test]
fn attributed_and_managed_lengths_count_payload_without_metadata() -> Result<(), EvalError> {
    let owner = Environment::global();
    let attributes = vec![ListElement {
        name: Some("names".into()),
        value: Sexp::String(vec![Some("a".into()), None, Some("".into())]),
    }];
    let mut values = [
        AtomicVector::Logical(vec![Some(true), None, Some(false)]),
        AtomicVector::Integer(vec![Some(1), None, Some(3)]),
        AtomicVector::Real(vec![1.0, r_ust_sexp::NA_REAL, f64::NAN]),
        AtomicVector::String(vec![Some("a".into()), None, Some("".into())]),
    ]
    .into_iter()
    .map(|values| Sexp::AttributedAtomic {
        values,
        attributes: attributes.clone(),
    })
    .collect::<Vec<_>>();
    for payload in [
        AggregateVector::Vector(vec![Sexp::Nil; 3]),
        AggregateVector::Expression(vec![Sexp::Symbol("unbound".into()); 3]),
    ] {
        values.push(Sexp::AttributedAggregate {
            values: payload,
            attributes: attributes.clone(),
        });
    }
    for raw in values {
        assert_eq!(value_len(&raw)?, 3);
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        assert_eq!(value_len(&Sexp::Managed(value))?, 3);
    }
    let empty = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![]),
        attributes: vec![ListElement {
            name: Some("note".into()),
            value: Sexp::Integer(vec![Some(1)]),
        }],
    };
    assert_eq!(value_len(&empty)?, 0);
    assert_eq!(
        value_len(&Sexp::Managed(ValueObject::import(
            &owner,
            &empty,
            ValueImport::Evaluated
        )?))?,
        0
    );
    Ok(())
}

#[test]
fn managed_aggregate_length_does_not_force_payload_promises() -> Result<(), EvalError> {
    let owner = Environment::global();
    let promise = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::Promise {
            state: promise.clone(),
        }]),
        attributes: vec![],
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    assert_eq!(value_len(&Sexp::Managed(value))?, 1);
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    Ok(())
}

#[test]
fn managed_environment_public_length_counts_bindings() -> Result<(), EvalError> {
    let owner = Environment::global();
    let frame = owner.child()?;
    frame.insert("a".into(), &Sexp::Nil)?;
    frame.insert("b".into(), &Sexp::Nil)?;
    let value = ValueObject::import(&owner, &Sexp::Environment(frame), ValueImport::Explicit)?;
    assert_eq!(
        crate::environment_values::length(&Sexp::Managed(value))?,
        Sexp::Integer(vec![Some(2)])
    );
    Ok(())
}
