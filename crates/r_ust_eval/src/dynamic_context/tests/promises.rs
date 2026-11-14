use super::*;
use r_ust_sexp::{EvaluationSessionId, PromiseState};

#[test]
fn promise_force_preserves_live_contexts() {
    let mut runtime = RuntimeState::new(Default::default());
    let outer = closure(&runtime.global, Sexp::Nil);
    host_closure(&mut runtime, &outer, |runtime, _| {
        let active = snapshot(runtime);
        let promise = Sexp::Promise {
            state: PromiseState::new(
                &expression("return(42L)"),
                &runtime.current,
                runtime.control.origin.clone(),
                true,
            )?,
        };
        let wrapper = closure(&runtime.global, Sexp::Nil);
        host_closure(runtime, &wrapper, |runtime, _| {
            let before = snapshot(runtime);
            let result = runtime.force_value(promise);
            let EvaluationOrigin::Frame(target) = active.2 else {
                panic!("active return required")
            };
            assert_eq!(
                result,
                Err(EvalError::Transfer(ControlTransfer::Return(
                    target,
                    Sexp::Integer(vec![Some(42)])
                )))
            );
            assert_eq!(snapshot(runtime), before);
            assert_eq!(runtime.control.contexts.len(), 2);
            Ok(EvalFlow::Value(Sexp::Nil))
        })?;
        assert_eq!(snapshot(runtime), active);
        Ok(EvalFlow::Value(Sexp::Nil))
    })
    .unwrap();
    assert!(runtime.control.contexts.is_empty());
}

#[test]
fn ineligible_origin_cannot_borrow_matching_environment() {
    let mut runtime = RuntimeState::new(Default::default());
    let function = closure(&runtime.global, Sexp::Nil);
    let mut inactive = EvaluationOrigin::Global;
    host_closure(&mut runtime, &function, |runtime, _| {
        inactive = runtime.control.origin.clone();
        Ok(EvalFlow::Value(Sexp::Nil))
    })
    .unwrap();
    for origin in [
        EvaluationOrigin::Global,
        EvaluationOrigin::Session(EvaluationSessionId::default()),
        inactive,
    ] {
        host_closure(&mut runtime, &function, |runtime, _| {
            runtime.with_loop(|runtime, _| {
                let before = snapshot(runtime);
                for source in ["break", "next", "return(42L)"] {
                    let value = Sexp::Promise {
                        state: PromiseState::new(
                            &expression(source),
                            &runtime.current,
                            origin.clone(),
                            true,
                        )?,
                    };
                    let message = if source.starts_with("return") {
                        "no function to return from, jumping to top level"
                    } else {
                        "no loop for break/next, jumping to top level"
                    };
                    assert_eq!(
                        runtime.force_value(value),
                        Err(EvalError::Control(message.into()))
                    );
                    assert_eq!(snapshot(runtime), before);
                }
                Ok(EvalFlow::Value(Sexp::Nil))
            })
        })
        .unwrap();
        assert!(runtime.control.contexts.is_empty());
    }
}

#[test]
fn eligible_session_matches_environment_across_new_return_identity() {
    let mut runtime = RuntimeState::new(Default::default());
    let saved_origin = runtime.control.origin.clone();
    let global = runtime.global.clone();
    let value = Sexp::Promise {
        state: PromiseState::new(&Sexp::Break, &global, saved_origin, true).unwrap(),
    };
    runtime
        .with_loop(|runtime, id| {
            let selected = entry(runtime, global);
            runtime.with_return_context(selected, |runtime| {
                let before = snapshot(runtime);
                assert_eq!(
                    runtime.force_value(value),
                    Err(EvalError::Transfer(ControlTransfer::Break(id)))
                );
                assert_eq!(snapshot(runtime), before);
                Ok(EvalFlow::Value(Sexp::Nil))
            })
        })
        .unwrap();
    assert!(runtime.control.contexts.is_empty());
}
