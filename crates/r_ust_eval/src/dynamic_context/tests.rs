use super::*;
use crate::{ClosureEnvironment, Frontend};

fn expression(source: &str) -> Sexp {
    crate::Parser
        .parse_all(source)
        .unwrap()
        .expressions
        .pop()
        .unwrap()
}

fn closure(environment: &Environment, body: Sexp) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(body),
        environment: ClosureEnvironment::Captured(environment.clone()),
    }
}

fn host_closure(
    runtime: &mut RuntimeState,
    callable: &Sexp,
    evaluate: impl FnOnce(&mut RuntimeState, &Sexp) -> Result<EvalFlow, EvalError>,
) -> Result<EvalFlow, EvalError> {
    let invocation = runtime.prepare_closure_call(callable.clone(), Sexp::Nil, &[])?;
    runtime.with_closure_context(invocation, evaluate)
}

fn entry(runtime: &RuntimeState, execution: Environment) -> ReturnContext {
    ReturnContext {
        execution,
        sysparent: runtime.current.clone(),
        callable: Sexp::Nil,
        arguments: Vec::new(),
        call: Sexp::Nil,
        generic: false,
    }
}

fn snapshot(runtime: &RuntimeState) -> (Environment, Environment, EvaluationOrigin, Vec<Context>) {
    (
        runtime.current.clone(),
        runtime.global.clone(),
        runtime.control.origin.clone(),
        runtime.control.contexts.clone(),
    )
}

mod exits;
mod promises;
mod selection;

#[test]
fn same_environment_eval_loop_target_survives_return_context() {
    for expression in [Sexp::Break, Sexp::Next] {
        let mut runtime = RuntimeState::new(Default::default());
        let initial = snapshot(&runtime);
        let wrapper = closure(&runtime.global, Sexp::Nil);
        runtime
            .with_loop(|runtime, loop_id| {
                let before = snapshot(runtime);
                let selected = runtime.current.clone();
                let result = host_closure(runtime, &wrapper, |runtime, _| {
                    runtime.eval_selected(entry(runtime, selected), &expression)
                });
                let expected = match expression {
                    Sexp::Break => ControlTransfer::Break(loop_id),
                    _ => ControlTransfer::Next(loop_id),
                };
                assert_eq!(result, Err(EvalError::Transfer(expected)));
                assert_eq!(snapshot(runtime), before);
                Ok(EvalFlow::Value(Sexp::Nil))
            })
            .unwrap();
        assert_eq!(snapshot(&runtime), initial);
    }
}

#[test]
fn different_environment_eval_cannot_target_outer_loop() {
    let mut runtime = RuntimeState::new(Default::default());
    for expression in [Sexp::Break, Sexp::Next] {
        runtime
            .with_loop(|runtime, _| {
                let selected = runtime.current.child()?;
                assert_eq!(
                    runtime.eval_selected(entry(runtime, selected), &expression),
                    Err(EvalError::Control(
                        "no loop for break/next, jumping to top level".into()
                    ))
                );
                Ok(EvalFlow::Value(Sexp::Nil))
            })
            .unwrap();
        assert!(runtime.control.contexts.is_empty());
    }
}

#[test]
fn selected_return_catches_own_return() {
    let mut runtime = RuntimeState::new(Default::default());
    let function = closure(&runtime.global, Sexp::Nil);
    let wrapper = function.clone();
    let result = host_closure(&mut runtime, &function, |runtime, _| {
        let outer = snapshot(runtime);
        let selected = runtime.current.clone();
        let result = host_closure(runtime, &wrapper, |runtime, _| {
            runtime.eval_selected(entry(runtime, selected), &expression("return(22L)"))
        })?;
        assert_eq!(result, EvalFlow::Value(Sexp::Integer(vec![Some(22)])));
        assert_eq!(snapshot(runtime), outer);
        Ok(EvalFlow::Value(Sexp::Integer(vec![Some(33)])))
    })
    .unwrap();
    assert_eq!(result, EvalFlow::Value(Sexp::Integer(vec![Some(33)])));
    assert!(runtime.control.contexts.is_empty());
}
