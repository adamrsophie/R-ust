use super::{drained, evaluate, global, integer, last};
use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::{
    CallArgument, EvaluationEnvironmentId, EvaluationOrigin, EvaluationSessionId, PromiseState,
    Sexp,
};

#[test]
fn promise_branch_uses_recorded_environment_once_and_preserves_cache() {
    let mut frontend = Frontend::default();
    let root = global(&mut frontend);
    let recorded = root.child().unwrap();
    let selected = root.child().unwrap();
    recorded.insert("x".into(), &integer(31)).unwrap();
    selected.insert("x".into(), &integer(99)).unwrap();
    let expression = Sexp::Block(vec![
        Sexp::Call {
            function: Box::new(Sexp::Symbol("cat".into())),
            arguments: vec![CallArgument::positional(Sexp::String(vec![Some(
                "ONCE".into(),
            )]))],
        },
        Sexp::Symbol("x".into()),
    ]);
    let state = PromiseState::new(&expression, &recorded, EvaluationOrigin::Global, true).unwrap();
    let mut call = evaluate(
        Sexp::Promise {
            state: state.clone(),
        },
        &selected,
    );
    let Sexp::Call { arguments, .. } = &mut call else {
        panic!("eval call")
    };
    arguments[2].value = Sexp::Call {
        function: Box::new(Sexp::Primitive(r_ust_sexp::PrimitiveId::Invisible)),
        arguments: vec![CallArgument::positional(Sexp::Nil)],
    };
    assert_eq!(frontend.runtime_mut().eval(&call).unwrap(), integer(31));
    assert!(frontend.runtime().is_visible());
    recorded.insert("x".into(), &integer(32)).unwrap();
    assert_eq!(frontend.runtime_mut().eval(&call).unwrap(), integer(31));
    assert!(frontend.runtime().is_visible());
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("ONCE".into())]
    );
    assert!(state.environment().unwrap().is_none());
    drained(&mut frontend);
}

#[test]
fn detached_promises_do_not_gain_the_selected_eval_return_or_loop_origin() {
    for origin in [
        EvaluationOrigin::Global,
        EvaluationOrigin::Session(EvaluationSessionId::default()),
        EvaluationOrigin::Frame(EvaluationEnvironmentId(u64::MAX)),
    ] {
        for (expression, message) in [
            (
                Sexp::Break,
                "control-flow error: no loop for break/next, jumping to top level",
            ),
            (
                Sexp::Next,
                "control-flow error: no loop for break/next, jumping to top level",
            ),
            (
                Sexp::Call {
                    function: Box::new(Sexp::Symbol("return".into())),
                    arguments: vec![CallArgument::positional(integer(88))],
                },
                "control-flow error: no function to return from, jumping to top level",
            ),
        ] {
            let mut frontend = Frontend::default();
            let frame = global(&mut frontend);
            let state = PromiseState::new(&expression, &frame, origin.clone(), true).unwrap();
            let payload = Sexp::Expression {
                names_present: false,
                elements: vec![r_ust_sexp::ListElement {
                    name: None,
                    value: Sexp::Promise { state },
                }],
            };
            frame.insert("payload".into(), &payload).unwrap();
            let error = frontend
                .eval_source("for(i in 1L){eval(payload,globalenv());cat('BAD')}")
                .unwrap_err();
            assert_eq!(error.to_string(), message);
            assert!(frontend.runtime_mut().take_output().is_empty());
            drained(&mut frontend);
        }
    }
}

#[test]
fn live_and_expired_source_promises_keep_their_original_control_targets() {
    let mut frontend = Frontend::default();
    assert_eq!(
        last(
            &mut frontend,
            "f<-function(x)evalq(x);(function(){f(return(42L));99L})()"
        ),
        integer(42)
    );
    assert_eq!(
        last(
            &mut frontend,
            "for(i in 1L){f(break);stop('UNREACHED')};42L"
        ),
        integer(42)
    );
    last(
        &mut frontend,
        "maker<-function(x)function()evalq(x);p<-(function()maker(return(42L)))()",
    );
    assert_eq!(
        frontend.eval_source("p()").unwrap_err().to_string(),
        "control-flow error: no function to return from, jumping to top level"
    );
    drained(&mut frontend);
}
