use super::*;
use crate::RuntimeState;
use r_ust_sexp::{EvaluationOrigin, Force, PromiseState};

#[test]
fn language_members_sdk_factor_indices_use_integer_codes() -> Result {
    let call = language("f(a)");
    for (code, expected) in [(Some(1), Some("f")), (Some(2), Some("a")), (None, None)] {
        let subscript = Sexp::Factor {
            codes: vec![code],
            levels: vec!["first".into(), "second".into()],
            ordered: false,
        };
        assert_eq!(
            extract(&call, &subscript)?,
            expected.map_or(Sexp::Nil, |name| Sexp::Symbol(name.into()))
        );
    }
    Ok(())
}

#[test]
fn language_members_sdk_preserves_promises_missing_cells_and_aliases() -> Result {
    let runtime = RuntimeState::default();
    let state = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &runtime.global,
        EvaluationOrigin::Global,
        true,
    )?;
    let promise = Sexp::Promise {
        state: state.clone(),
    };
    let original = Sexp::Call {
        function: Box::new(Sexp::Symbol("f".into())),
        arguments: vec![
            CallArgument::named("payload", promise.clone()),
            CallArgument::positional(Sexp::MissingValue),
        ],
    };
    let alias = original.clone();
    let result = assign(&original, &index(1), &Sexp::Symbol("g".into()))?;
    assert_eq!(original, alias);
    assert_eq!(extract(&result, &index(2))?, promise);
    assert_eq!(extract(&result, &index(3))?, Sexp::MissingValue);
    let inserted = assign(&language("f()"), &name(Some("p")), &promise)?;
    assert_eq!(extract(&inserted, &name(Some("p")))?, promise);
    assert!(matches!(state.begin_force()?, Force::Evaluate(_)));
    Ok(())
}

#[test]
fn language_members_sdk_exported_graph_payload_outlives_runtime() -> Result {
    let result;
    {
        let runtime = RuntimeState::default();
        let payload = runtime.global.child()?;
        payload.insert("x".into(), &index(42))?;
        result = assign(
            &language("f()"),
            &name(Some("payload")),
            &Sexp::Environment(payload),
        )?;
    }
    let Sexp::Environment(payload) = extract(&result, &name(Some("payload")))? else {
        panic!("environment payload")
    };
    assert_eq!(payload.get_local("x")?, Some(index(42)));
    Ok(())
}
