use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::{
    AtomicVector, EvaluationOrigin, Force, ListElement, PromiseState, ValueImport, ValueObject,
};

#[test]
fn attributed_and_managed_data_self_evaluate_without_metadata_loss() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(7)]),
        attributes: vec![ListElement {
            name: Some("note".into()),
            value: Sexp::String(vec![Some("kept".into())]),
        }],
    };
    assert_eq!(runtime.eval(&raw)?, raw);
    let value = ValueObject::import(&runtime.global, &raw, ValueImport::Evaluated)?;
    let Sexp::Managed(returned) = runtime.eval(&Sexp::Managed(value.clone()))? else {
        panic!("persistent value")
    };
    assert!(returned.same_object(&value));
    assert_eq!(returned.snapshot()?, raw);
    Ok(())
}

#[test]
fn managed_symbols_use_runtime_lookup_in_both_import_modes() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    runtime
        .global
        .insert("x".into(), &Sexp::Integer(vec![Some(9)]))?;
    for mode in [ValueImport::Code, ValueImport::Evaluated] {
        let value = ValueObject::import(&runtime.global, &Sexp::Symbol("x".into()), mode)?;
        assert_eq!(
            runtime.eval(&Sexp::Managed(value))?,
            Sexp::Integer(vec![Some(9)])
        );
    }
    Ok(())
}

#[test]
fn managed_explicit_promise_uses_existing_force_and_cache() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let expected = Sexp::Integer(vec![Some(11)]);
    let promise = PromiseState::new(&expected, &runtime.global, EvaluationOrigin::Global, true)?;
    let value = ValueObject::import(
        &runtime.global,
        &Sexp::Promise {
            state: promise.clone(),
        },
        ValueImport::Explicit,
    )?;
    assert_eq!(runtime.eval(&Sexp::Managed(value))?, expected);
    let Force::Cached(cached) = promise.begin_force()? else {
        panic!("forced promise cache")
    };
    assert_eq!(cached, expected);
    Ok(())
}
