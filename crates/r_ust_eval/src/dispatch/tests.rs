use crate::closure_calls::ClosureInvocation;
use crate::dynamic_context::ContextKind;
use crate::{CallArgument, ControlTransfer, EvalError, EvalFlow, Frontend, RuntimeState, Sexp};

fn expression(source: &str) -> Sexp {
    crate::Parser
        .parse_all(source)
        .unwrap()
        .expressions
        .remove(0)
}

fn invocation(runtime: &mut RuntimeState, name: &str, source: &str) -> ClosureInvocation {
    let call = expression(source);
    let Sexp::Call { arguments, .. } = &call else {
        panic!("call required")
    };
    runtime
        .prepare_closure_call(
            runtime.raw_lookup(name).unwrap().unwrap(),
            call.clone(),
            arguments,
        )
        .unwrap()
}

#[test]
fn dispatched_bindings_and_method_context_retain_promarg_identity() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("g<-function(x,...)UseMethod('g');g.integer<-function(x,...)environment()")
        .unwrap();
    let runtime = frontend.runtime_mut();
    let generic = invocation(runtime, "g", "g(1L,z=2L)");
    let original = generic.arguments.clone();
    assert!(matches!(original[0].value, Sexp::Promise { .. }));
    let result = runtime
        .with_closure_context(generic, |runtime, _| {
            let saved = runtime.control.contexts.last().unwrap().clone();
            assert_eq!(saved.arguments, original);
            let result = runtime.eval_use_method(
                &[CallArgument::positional(Sexp::String(vec![Some(
                    "g".into(),
                )]))],
                &expression("UseMethod('g')"),
            );
            let Err(EvalError::Transfer(ControlTransfer::Return(
                target,
                Sexp::Environment(method),
            ))) = &result
            else {
                panic!("actual method must return its execution frame: {result:?}")
            };
            assert_eq!(ContextKind::Return(*target), saved.kind);
            assert_eq!(method.get_local("x")?, Some(original[0].value.clone()));
            let Some(Sexp::Dots(dots)) = method.get_local("...")? else {
                panic!("method dots required")
            };
            assert_eq!(dots[0].value, original[1].value);
            assert_eq!(runtime.control.contexts.last().unwrap(), &saved);
            // The existing invocation hook observes the same production method-frame owner directly.
            let method_call = ClosureInvocation {
                callable: runtime.raw_lookup("g.integer")?.unwrap(),
                arguments: original.clone(),
                call: expression("g.integer(1L,z=2L)"),
                sysparent: saved.sysparent,
                bindings: vec![],
            };
            runtime.with_closure_context(method_call, |runtime, _| {
                let active = runtime.control.contexts.last().unwrap();
                assert_eq!(active.arguments, original);
                assert_eq!(
                    active.execution.get_local("x")?,
                    Some(active.arguments[0].value.clone())
                );
                let Some(Sexp::Dots(dots)) = active.execution.get_local("...")? else {
                    panic!("active dots required")
                };
                assert_eq!(dots[0].value, active.arguments[1].value);
                Ok(EvalFlow::Value(Sexp::Nil))
            })?;
            result
        })
        .unwrap();
    assert!(matches!(result, EvalFlow::Value(Sexp::Environment(_))));
    assert!(runtime.control.contexts.is_empty());
}

fn fail_output(_: crate::OutputEvent) -> std::io::Result<()> {
    Err(std::io::Error::other("live generic output failure"))
}

fn fail_input() -> std::io::Result<String> {
    Err(std::io::Error::other("live generic input failure"))
}

#[test]
fn generic_flag_restores_before_all_method_exits_propagate() {
    for (kind, body) in [
        ("value", "invisible(23L)"),
        ("return", "y"),
        ("language", "unknown_method_value"),
        ("output", "print.default(x)"),
        ("input", "readline()"),
        ("quit", "q('no',7L,FALSE)"),
    ] {
        let mut frontend = Frontend::default();
        frontend.eval_source(&format!("outer<-function()NULL;g<-function(x,y)UseMethod('g');g.integer<-function(x,y){body}")).unwrap();
        let runtime = frontend.runtime_mut();
        if kind == "output" {
            runtime.set_output_sink(fail_output);
        }
        runtime.set_input_reader(fail_input);
        runtime.set_session_context(vec![], true);
        let outer = invocation(runtime, "outer", "outer()");
        let result = runtime.with_closure_context(outer, |runtime, _| {
            let parent_kind = runtime.control.contexts.last().unwrap().kind;
            let generic = invocation(runtime, "g", "g(1L,return(17L))");
            runtime.with_closure_context(generic, |runtime, _| {
                let saved = runtime.control.contexts.last().unwrap().clone();
                assert!(!saved.generic);
                let result = runtime.eval_use_method(
                    &[CallArgument::positional(Sexp::String(vec![Some(
                        "g".into(),
                    )]))],
                    &expression("UseMethod('g')"),
                );
                assert_eq!(
                    runtime.control.contexts.last().unwrap(),
                    &saved,
                    "{kind}: generic state changed before propagation"
                );
                assert_eq!(runtime.control.contexts.len(), 2);
                match (kind, &result) {
                    ("value", Err(EvalError::Transfer(ControlTransfer::Return(target, value)))) => {
                        assert_eq!(ContextKind::Return(*target), saved.kind);
                        assert_eq!(value, &Sexp::Integer(vec![Some(23)]));
                        assert!(!runtime.visible);
                    }
                    (
                        "return",
                        Err(EvalError::Transfer(ControlTransfer::Return(target, value))),
                    ) => {
                        assert_eq!(ContextKind::Return(*target), parent_kind);
                        assert_eq!(value, &Sexp::Integer(vec![Some(17)]));
                    }
                    ("language", Err(EvalError::Signaled(error))) => {
                        assert_eq!(
                            error.as_ref(),
                            &EvalError::UndefinedSymbol("unknown_method_value".into())
                        )
                    }
                    ("output", Err(EvalError::Output(message))) => {
                        assert_eq!(message, "live generic output failure")
                    }
                    ("input", Err(EvalError::Input(message))) => {
                        assert_eq!(message, "live generic input failure")
                    }
                    ("quit", Err(EvalError::Quit(request))) => assert_eq!(request.status, 7),
                    _ => panic!("{kind}: incorrect method exit {result:?}"),
                }
                result
            })
        });
        match kind {
            "value" => assert_eq!(result, Ok(EvalFlow::Value(Sexp::Integer(vec![Some(23)])))),
            "return" => assert_eq!(result, Ok(EvalFlow::Value(Sexp::Integer(vec![Some(17)])))),
            _ => assert!(result.is_err()),
        }
        assert!(runtime.control.contexts.is_empty(), "{kind}");
        assert_eq!(runtime.current, runtime.global);
    }
}
