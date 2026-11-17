use super::{contains, scalar_missing};
use crate::{EvalError, Sexp};
use r_ust_sexp::{
    AggregateVector, AtomicVector, Environment, EvaluationOrigin, Force, ListElement, PromiseState,
    ValueImport, ValueObject,
};

#[test]
fn typed_and_managed_missing_values_ignore_metadata_and_unused_recursive_argument(
) -> Result<(), EvalError> {
    let owner = Environment::global();
    for payload in [
        AtomicVector::Logical(vec![None]),
        AtomicVector::Integer(vec![None]),
        AtomicVector::Real(vec![f64::NAN]),
        AtomicVector::String(vec![None]),
    ] {
        let raw = Sexp::AttributedAtomic {
            values: payload,
            attributes: vec![],
        };
        assert!(contains(&raw, &Sexp::Nil)?);
        assert!(scalar_missing(&raw)?);
        let managed = Sexp::Managed(ValueObject::import(&owner, &raw, ValueImport::Evaluated)?);
        assert!(contains(&managed, &Sexp::Nil)?);
        assert!(scalar_missing(&managed)?);
    }
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1)]),
        attributes: vec![ListElement {
            name: Some("note".into()),
            value: Sexp::Integer(vec![None]),
        }],
    };
    assert!(!contains(&raw, &Sexp::Nil)?);
    Ok(())
}

#[test]
fn managed_list_recursion_distinguishes_scalar_missing_and_repeated_child_aliases(
) -> Result<(), EvalError> {
    let owner = Environment::global();
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::Integer(vec![None, Some(1)])]),
        attributes: vec![],
    };
    let value = Sexp::Managed(ValueObject::import(&owner, &raw, ValueImport::Evaluated)?);
    assert!(!contains(&value, &Sexp::Logical(vec![Some(false)]))?);
    assert!(contains(&value, &Sexp::Logical(vec![Some(true)]))?);
    let child = ValueObject::import(
        &owner,
        &Sexp::Integer(vec![Some(7)]),
        ValueImport::Evaluated,
    )?;
    let aliases = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![child.transport()?, child.transport()?]),
        attributes: vec![],
    };
    let aliases = Sexp::Managed(ValueObject::import(
        &owner,
        &aliases,
        ValueImport::Evaluated,
    )?);
    assert!(!contains(&aliases, &Sexp::Logical(vec![Some(true)]))?);
    let promise = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    let promise_value = ValueObject::import(
        &owner,
        &Sexp::Promise {
            state: promise.clone(),
        },
        ValueImport::Explicit,
    )?;
    assert!(!scalar_missing(&Sexp::Managed(promise_value))?);
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    Ok(())
}
