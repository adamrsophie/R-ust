use super::*;
use crate::list_conversion::call_c;
use r_ust_sexp::{Environment, EvaluationOrigin, Force, PromiseState, ValueImport, ValueObject};

#[test]
fn managed_atomic_names_are_read_without_mutating_payload_or_metadata() -> Result<(), EvalError> {
    let owner = Environment::global();
    let raw = named(vec![Some(1), Some(2)], vec![Some("a"), None]);
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let alias = object.clone();
    let before = object.snapshot()?;
    for value in [raw, Sexp::Managed(object.clone())] {
        check(
            call_c(&[argument(Some("outer"), value)])?,
            Sexp::Integer(vec![Some(1), Some(2)]),
            Some(vec![Some("outer.a"), Some("outer.NA")]),
        )?;
    }
    assert!(object.same_object(&alias));
    assert_eq!(object.snapshot()?, before);
    drop(owner);
    check(
        call_c(&[argument(None, Sexp::Managed(alias))])?,
        Sexp::Integer(vec![Some(1), Some(2)]),
        Some(vec![Some("a"), None]),
    )?;
    Ok(())
}

#[test]
fn managed_control_storage_and_typed_logical_payload_preserve_selection() -> Result<(), EvalError> {
    let owner = Environment::global();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Logical(vec![Some(true), None]),
        attributes: vec![ListElement {
            name: Some("note".into()),
            value: Sexp::Integer(vec![Some(7)]),
        }],
    };
    for managed in [false, true] {
        let wrap = |value: &Sexp| -> Result<Sexp, EvalError> {
            Ok(if managed {
                Sexp::Managed(ValueObject::import(&owner, value, ValueImport::Evaluated)?)
            } else {
                value.clone()
            })
        };
        check(
            call_c(&[
                argument(Some("x"), wrap(&raw)?),
                argument(
                    Some("use.names"),
                    wrap(&Sexp::AttributedAtomic {
                        values: AtomicVector::Logical(vec![Some(false)]),
                        attributes: vec![],
                    })?,
                ),
            ])?,
            Sexp::Logical(vec![Some(true), None]),
            None,
        )?;
    }
    Ok(())
}

#[test]
fn unsupported_promise_storage_is_not_forced_by_default_concatenation() -> Result<(), EvalError> {
    let owner = Environment::global();
    let state = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &owner,
        EvaluationOrigin::Global,
        true,
    )?;
    let value = ValueObject::import(
        &owner,
        &Sexp::Promise {
            state: state.clone(),
        },
        ValueImport::Explicit,
    )?;
    assert_eq!(
        call_c(&[argument(None, Sexp::Managed(value))]),
        Err(EvalError::NotImplemented(
            "default concatenation of non-atomic storage"
        ))
    );
    assert!(matches!(state.begin_force()?, Force::Evaluate(_)));
    Ok(())
}
