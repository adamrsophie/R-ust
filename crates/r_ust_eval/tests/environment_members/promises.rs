use super::*;
use r_ust_sexp::{EvaluationOrigin, Force, PromiseState};

#[test]
fn computed_missing_is_forced_once_and_cached() {
    let mut frontend = Frontend::default();
    let source = global(frontend.runtime());
    let frame = source.child().unwrap();
    let state = PromiseState::new(
        &expression("{cat('ONCE');quote(expr=)}"),
        &source,
        EvaluationOrigin::Global,
        true,
    )
    .unwrap();
    frame
        .insert(
            "lazy".into(),
            &Sexp::Promise {
                state: state.clone(),
            },
        )
        .unwrap();
    bind(
        frontend.runtime_mut(),
        "e",
        Sexp::Environment(frame.clone()),
    );
    assert_eq!(frame.local_len().unwrap(), 1);
    assert_eq!(
        frame.get_local("lazy").unwrap(),
        Some(Sexp::Promise {
            state: state.clone()
        })
    );
    assert_eq!(state.environment().unwrap(), Some(source));
    assert!(frontend.runtime_mut().take_output().is_empty());

    assert_eq!(
        frontend.eval_source("e$lazy; e[['lazy']]").unwrap(),
        [Sexp::MissingValue, Sexp::MissingValue]
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("ONCE".into())]
    );
    let Force::Cached(value) = state.begin_force().unwrap() else {
        panic!("member promise was not cached")
    };
    assert_eq!(value, Sexp::MissingValue);
    assert_eq!(state.environment().unwrap(), None);
}

#[test]
fn promise_member_keeps_source_session_after_runtime_drop() {
    let (frame, source, state) = {
        let mut frontend = Frontend::default();
        frontend.eval_source("effect<-0L; sentinel<-11L").unwrap();
        let source = global(frontend.runtime());
        let frame = source.child().unwrap();
        let state = PromiseState::new(
            &expression("{effect<<-effect+1L;sentinel}"),
            &frame,
            EvaluationOrigin::Global,
            true,
        )
        .unwrap();
        frame
            .insert(
                "lazy".into(),
                &Sexp::Promise {
                    state: state.clone(),
                },
            )
            .unwrap();
        (frame, source, state)
    };
    let mut consumer = Frontend::default();
    consumer.eval_source("effect<-99L;sentinel<-77L").unwrap();
    bind(
        consumer.runtime_mut(),
        "e",
        Sexp::Environment(frame.clone()),
    );
    assert_eq!(state.environment().unwrap(), Some(frame));

    assert_eq!(
        consumer.eval_source("e[['lazy']]; e$lazy").unwrap(),
        [integer(11), integer(11)]
    );
    assert_eq!(source.get_local("effect").unwrap(), Some(integer(1)));
    assert_eq!(consumer.runtime().get("effect").unwrap(), Some(integer(99)));
    assert_eq!(consumer.eval_source("sentinel").unwrap(), [integer(77)]);
    let Force::Cached(value) = state.begin_force().unwrap() else {
        panic!("source promise was not cached")
    };
    assert_eq!(value, integer(11));
}

#[test]
fn member_promise_preserves_control_origin_and_restores_consumer() {
    let mut frontend = Frontend::default();
    let source = global(frontend.runtime());
    let frame = source.child().unwrap();
    let state = PromiseState::new(
        &expression("return(42L)"),
        &source,
        EvaluationOrigin::Global,
        true,
    )
    .unwrap();
    frame
        .insert("exit".into(), &Sexp::Promise { state })
        .unwrap();
    bind(frontend.runtime_mut(), "e", Sexp::Environment(frame));
    assert_eq!(
        frontend.eval_source("(function(){private<-9L;e$exit;99L})()"),
        Err(EvalError::Control(
            "no function to return from, jumping to top level".into()
        ))
    );
    assert_eq!(frontend.runtime().get("private").unwrap(), None);
    assert_eq!(frontend.eval_source("7L").unwrap(), [integer(7)]);
}
