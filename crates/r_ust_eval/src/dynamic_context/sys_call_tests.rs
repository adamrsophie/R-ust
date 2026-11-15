use super::*;
use crate::control::ControlState;
use r_ust_sexp::{canonical_actual_call, BinaryOp, CallArgument, Force, PromiseState};

fn call(name: &str) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: vec![],
    }
}
fn entry(execution: Environment, sysparent: Environment, call: Sexp) -> Result<Context, EvalError> {
    Ok(Context {
        kind: ContextKind::Return(EvaluationEnvironmentId(next_id()?)),
        execution,
        sysparent,
        call,
        callable: Sexp::Nil,
        handler_stack: None,
        restart_stack: None,
        cleanup: Default::default(),
        arguments: vec![],
        generic: false,
    })
}

#[test]
fn sys_call_sdk_selects_sysparent_matching_context_not_latest_function() -> Result<(), EvalError> {
    let runtime = RuntimeState::default();
    let first = runtime.global.child()?;
    let second = first.child()?;
    let observer = second.child()?;
    let mut control = ControlState::default();
    control.contexts = vec![
        entry(first.clone(), runtime.global.clone(), call("first"))?,
        entry(second.clone(), first.clone(), call("second"))?,
        entry(observer, first, call("sys.call"))?,
    ];
    assert_eq!(control.frame_call(0)?, call("first"));
    assert_eq!(control.frame_call(1)?, call("first"));
    assert_eq!(control.frame_call(-1)?, Sexp::Nil);
    for index in [2, -2, i32::MAX, i32::MIN] {
        assert_eq!(
            control.frame_call(index).map_err(EvalError::unhandled),
            Err(EvalError::Domain(
                "not that many frames on the stack".into()
            ))
        );
    }
    Ok(())
}

#[test]
fn sys_call_sdk_ignores_loop_depth_and_canonicalizes_nested_parser_nodes() -> Result<(), EvalError>
{
    let mut runtime = RuntimeState::default();
    let first = runtime.global.child()?;
    let observer = first.child()?;
    let original = Sexp::Call {
        function: Box::new(Sexp::Symbol("f".into())),
        arguments: vec![CallArgument::named(
            "x",
            Sexp::Binary {
                op: BinaryOp::Add,
                left: Box::new(Sexp::Integer(vec![Some(1)])),
                right: Box::new(Sexp::Integer(vec![Some(2)])),
            },
        )],
    };
    let mut loop_context = None;
    runtime.with_loop(|runtime, _| {
        loop_context = runtime.control.contexts.last().cloned();
        Ok(EvalFlow::Value(Sexp::Nil))
    })?;
    let loop_context =
        loop_context.ok_or_else(|| EvalError::Control("missing loop context".into()))?;
    let mut control = ControlState::default();
    control.contexts = vec![
        entry(first.clone(), runtime.global.clone(), original.clone())?,
        loop_context,
        entry(observer, first, call("sys.call"))?,
    ];
    assert_eq!(control.selected_depth(), 1);
    assert_eq!(
        control.frame_call(0)?,
        canonical_actual_call(&original).expect("canonical source")
    );
    assert_eq!(control.frame_call(1)?, control.frame_call(0)?);
    assert_eq!(control.frame_call(-1)?, Sexp::Nil);
    Ok(())
}

#[test]
fn sys_call_sdk_copies_call_structure_without_forcing_or_replacing_promise_identity(
) -> Result<(), EvalError> {
    let runtime = RuntimeState::default();
    let first = runtime.global.child()?;
    let observer = first.child()?;
    let promise = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &runtime.global,
        EvaluationOrigin::Global,
        true,
    )?;
    let original = Sexp::Call {
        function: Box::new(Sexp::Symbol("f".into())),
        arguments: vec![CallArgument::named(
            "payload",
            Sexp::Promise {
                state: promise.clone(),
            },
        )],
    };
    let mut control = ControlState::default();
    control.contexts = vec![
        entry(first.clone(), runtime.global.clone(), original.clone())?,
        entry(observer, first, call("sys.call"))?,
    ];
    let mut result = control.frame_call(0)?;
    let Sexp::Call {
        function,
        arguments,
    } = &mut result
    else {
        panic!("call")
    };
    assert_eq!(
        arguments[0].value,
        Sexp::Promise {
            state: promise.clone()
        }
    );
    *function = Box::new(Sexp::Symbol("changed".into()));
    arguments.clear();
    assert_eq!(control.frame_call(0)?, original);
    assert!(matches!(promise.begin_force()?, Force::Evaluate(_)));
    Ok(())
}

#[test]
fn sys_call_sdk_exported_call_keeps_graph_payload_after_context_and_runtime_drop(
) -> Result<(), EvalError> {
    let result;
    {
        let runtime = RuntimeState::default();
        let first = runtime.global.child()?;
        let observer = first.child()?;
        let payload = runtime.global.child()?;
        payload.insert("x".into(), &Sexp::Integer(vec![Some(42)]))?;
        let original = Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![CallArgument::positional(Sexp::Environment(payload))],
        };
        let mut control = ControlState::default();
        control.contexts = vec![
            entry(first.clone(), runtime.global.clone(), original)?,
            entry(observer, first, call("sys.call"))?,
        ];
        result = control.frame_call(0)?;
    }
    let Sexp::Call { arguments, .. } = result else {
        panic!("call")
    };
    let Sexp::Environment(payload) = &arguments[0].value else {
        panic!("graph payload")
    };
    assert_eq!(payload.get_local("x")?, Some(Sexp::Integer(vec![Some(42)])));
    Ok(())
}
