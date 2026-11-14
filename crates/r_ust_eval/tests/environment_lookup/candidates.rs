use super::*;
use r_ust_sexp::{EvaluationOrigin, PromiseState};

#[test]
fn function_candidates_skip_scalars_but_stop_at_missing_and_force_once() {
    let global = Environment::global();
    let frame = global.child().unwrap();
    let mut runtime = RuntimeState::default();
    for value in [
        integer(7),
        Sexp::MissingArg("sum".into()),
        Sexp::MissingValue,
    ] {
        frame.insert("sum".into(), &value).unwrap();
        let read = invoke(&mut runtime, &closure(&frame, Sexp::Symbol("sum".into())));
        let call = invoke(&mut runtime, &closure(&frame, expression("sum(2L, 3L)")));
        if value == integer(7) {
            assert_eq!(read.unwrap(), integer(7));
            assert_eq!(call.unwrap(), integer(5));
        } else {
            assert_eq!(read, Err(EvalError::MissingArgument("sum".into())));
            assert_eq!(call, Err(EvalError::MissingArgument("sum".into())));
        }
    }
    for (body, expected) in [
        ("{hits <<- hits + 1L; 7L}", integer(5)),
        ("{hits <<- hits + 1L; function(...) 17L}", integer(17)),
    ] {
        global.insert("hits".into(), &integer(0)).unwrap();
        let state = PromiseState::new(
            &expression(body),
            &global,
            EvaluationOrigin::default(),
            true,
        )
        .unwrap();
        frame
            .insert("sum".into(), &Sexp::Promise { state })
            .unwrap();
        assert!(matches!(
            frame.get_local("sum").unwrap(),
            Some(Sexp::Promise { .. })
        ));
        assert_eq!(frame.local_len().unwrap(), 1);
        assert_eq!(global.get_local("hits").unwrap(), Some(integer(0)));
        for _ in 0..2 {
            assert_eq!(
                invoke(&mut runtime, &closure(&frame, expression("sum(2L, 3L)"))).unwrap(),
                expected
            );
        }
        assert_eq!(global.get_local("hits").unwrap(), Some(integer(1)));
    }
    for body in [Sexp::MissingValue, Sexp::Symbol("absent".into())] {
        let state = PromiseState::new(&body, &global, EvaluationOrigin::default(), true).unwrap();
        frame
            .insert("sum".into(), &Sexp::Promise { state })
            .unwrap();
        let expected = if body == Sexp::MissingValue {
            EvalError::MissingArgument("sum".into())
        } else {
            EvalError::UndefinedSymbol("absent".into())
        };
        assert_eq!(
            invoke(&mut runtime, &closure(&frame, expression("sum(2L)"))),
            Err(expected)
        );
    }
}

#[test]
fn raw_runtime_get_does_not_force_stored_promises() {
    let mut runtime = RuntimeState::default();
    let global = root(&runtime, EnvironmentRole::Global);
    let state = PromiseState::new(
        &expression("effect <- 19L"),
        &global,
        EvaluationOrigin::default(),
        true,
    )
    .unwrap();
    global
        .insert("pending".into(), &Sexp::Promise { state })
        .unwrap();
    assert!(matches!(
        runtime.get("pending").unwrap(),
        Some(Sexp::Promise { .. })
    ));
    assert_eq!(global.local_len().unwrap(), 1);
    assert_eq!(runtime.get("effect").unwrap(), None);
    assert_eq!(
        runtime.eval(&Sexp::Symbol("pending".into())).unwrap(),
        integer(19)
    );
    assert_eq!(runtime.get("effect").unwrap(), Some(integer(19)));
}
