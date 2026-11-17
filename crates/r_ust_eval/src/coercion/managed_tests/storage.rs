use super::managed;
use crate::coercion::{atomic, lists, storage, strings::BlankPolicy};
use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::{
    AggregateVector, Environment, EvaluationOrigin, Force, PromiseState, ValueImport, ValueObject,
};

#[test]
fn attributed_aggregate_projection_keeps_child_identity_and_ownership() -> Result<(), EvalError> {
    let owner = Environment::global();
    let child = ValueObject::import(
        &owner,
        &Sexp::Integer(vec![Some(8)]),
        ValueImport::Evaluated,
    )?;
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![child.transport()?]),
        attributes: vec![ListElement {
            name: Some("class".into()),
            value: Sexp::String(vec![Some("box".into())]),
        }],
    };
    let input = managed(&owner, &raw)?;
    let count = child.semantic_reference_count()?;
    {
        let projection = storage::read(&input)?;
        let Sexp::Vector(values) = projection.as_ref() else {
            panic!("expected vector storage")
        };
        let Sexp::Managed(actual) = &values[0] else {
            panic!("child must remain managed")
        };
        assert!(actual.same_object(&child));
        assert_eq!(child.semantic_reference_count()?, count);
        assert_eq!(
            lists::convert(&input, (true, BlankPolicy::C), |_| Ok(()))?,
            Sexp::Integer(vec![Some(8)])
        );
    }
    assert_eq!(child.semantic_reference_count()?, count);
    let expression = Sexp::AttributedAggregate {
        values: AggregateVector::Expression(vec![child.transport()?]),
        attributes: vec![],
    };
    let input = managed(&owner, &expression)?;
    let projection = storage::read(&input)?;
    let Sexp::Expression { elements, .. } = projection.as_ref() else {
        panic!("expected expression storage")
    };
    let Sexp::Managed(actual) = &elements[0].value else {
        panic!("child must remain managed")
    };
    assert!(actual.same_object(&child));
    assert!(lists::vectorizable(&input)?);
    Ok(())
}

#[test]
fn explicit_runtime_roles_are_not_forced_or_coerced_as_atomic_storage() -> Result<(), EvalError> {
    let owner = Environment::global();
    let state = PromiseState::new(
        &Sexp::Symbol("must_not_run".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    let promise = Sexp::Promise {
        state: state.clone(),
    };
    let input = managed(&owner, &Sexp::Vector(vec![promise.clone()]))?;
    assert!(lists::convert(&input, (true, BlankPolicy::C), |_| Ok(())).is_err());
    assert!(matches!(state.begin_force()?, Force::Evaluate(_)));
    assert_eq!(atomic::scalar_integer(&promise, BlankPolicy::C).0, None);
    let environment = Sexp::Environment(owner.clone());
    assert!(atomic::integers(&environment, BlankPolicy::C).is_err());
    assert_eq!(crate::internal::as_logical(&environment), None);
    assert!(
        ValueObject::import(&owner, &Sexp::Integer(vec![Some(1)]), ValueImport::Explicit).is_err()
    );
    Ok(())
}
