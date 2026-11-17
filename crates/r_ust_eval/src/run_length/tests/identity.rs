use super::*;
use r_ust_sexp::{EvaluationOrigin, Force, PromiseState};

#[test]
fn inverse_run_length_reuses_list_child_handles_after_inputs_drop() -> Result<(), EvalError> {
    let owner = Environment::global();
    let child = ValueObject::import(
        &owner,
        &Sexp::Integer(vec![Some(7)]),
        ValueImport::Evaluated,
    )?;
    let raw = Sexp::Vector(vec![child.transport()?, Sexp::Integer(vec![Some(8)])]);
    let input = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?.transport()?;
    let result = decoded(Sexp::Integer(vec![Some(2), Some(0)]), input)?;
    let Sexp::Vector(values) = &result else {
        panic!("list result")
    };
    let (Sexp::Managed(a), Sexp::Managed(b)) = (&values[0], &values[1]) else {
        panic!("child transports")
    };
    assert!(a.same_object(&child) && b.same_object(&child));
    drop(raw);
    drop(child);
    drop(owner);
    assert_eq!(a.snapshot()?, Sexp::Integer(vec![Some(7)]));
    Ok(())
}

#[test]
fn inverse_run_length_keeps_expression_kind_and_never_forces_children() -> Result<(), EvalError> {
    let owner = Environment::global();
    let promise = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    let raw = Sexp::Expression {
        elements: vec![ListElement {
            name: Some("x".into()),
            value: Sexp::Promise {
                state: promise.clone(),
            },
        }],
        names_present: true,
    };
    let input = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?.transport()?;
    let result = decoded(Sexp::Integer(vec![Some(2)]), input)?;
    let Sexp::Expression {
        elements,
        names_present,
    } = result
    else {
        panic!("expression result")
    };
    assert!(!names_present);
    assert_eq!(elements.len(), 2);
    assert_eq!(elements[0].value, elements[1].value);
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    Ok(())
}

#[test]
fn run_length_one_managed_list_retains_unforced_child_identity() -> Result<(), EvalError> {
    let owner = Environment::global();
    let promise = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    let raw = Sexp::Vector(vec![Sexp::Promise {
        state: promise.clone(),
    }]);
    let input = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?.transport()?;
    let result = encoded(input)?;
    let view = result.ordinary_list().unwrap();
    let Sexp::Vector(values) = &view.elements[1].value else {
        panic!("list values")
    };
    assert_eq!(
        values[0],
        Sexp::Promise {
            state: promise.clone()
        }
    );
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    Ok(())
}
