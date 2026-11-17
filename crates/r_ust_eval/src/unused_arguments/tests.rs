use crate::{CallArgument, EvalError, RuntimeState, Sexp};
use r_ust_sexp::{
    AtomicVector, EvaluationOrigin, Force, ListElement, PromiseState, ValueImport, ValueObject,
};

#[test]
fn unused_typed_argument_uses_simple_deparse_without_names() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let value = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(7)]),
        attributes: vec![ListElement {
            name: Some("names".into()),
            value: Sexp::String(vec![Some("label".into())]),
        }],
    };
    let value = ValueObject::import(&runtime.global, &value, ValueImport::Evaluated)?;
    let argument = CallArgument::named("x", Sexp::Managed(value));
    assert_eq!(
        runtime.render_unused_arguments(&[&argument])?,
        "unused argument (x = 7)"
    );
    Ok(())
}

#[test]
fn unused_managed_supplied_promise_is_peeled_without_forcing() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let promise = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &runtime.global,
        EvaluationOrigin::Global,
        true,
    )?;
    let value = ValueObject::import(
        &runtime.global,
        &Sexp::Promise {
            state: promise.clone(),
        },
        ValueImport::Explicit,
    )?;
    let argument = CallArgument::named("x", Sexp::Managed(value));
    assert_eq!(
        runtime.render_unused_arguments(&[&argument])?,
        "unused argument (x = never_force)"
    );
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    Ok(())
}
