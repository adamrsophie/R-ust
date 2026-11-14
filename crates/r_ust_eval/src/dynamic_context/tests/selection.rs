use super::*;

#[test]
fn selected_return_context_counts_as_function() {
    let mut runtime = RuntimeState::new(Default::default());
    let function = closure(&runtime.global, expression("41L"));
    let wrapper = closure(&runtime.global, expression("42L"));
    let observer = closure(&runtime.global, Sexp::Nil);
    host_closure(&mut runtime, &function, |runtime, _| {
        let execution = runtime.current.clone();
        host_closure(runtime, &wrapper, |runtime, _| {
            let wrapper_frame = runtime.current.clone();
            let selected = ReturnContext {
                execution: execution.clone(),
                sysparent: wrapper_frame.clone(),
                callable: Sexp::Primitive(r_ust_sexp::PrimitiveId::Quote),
                arguments: Vec::new(),
                call: Sexp::Nil,
                generic: false,
            };
            runtime.with_return_context(selected, |runtime| {
                host_closure(runtime, &observer, |runtime, _| {
                    assert_eq!(runtime.control.selected_depth(), 3);
                    assert_eq!(runtime.control.parent_number(1, &runtime.global), 2);
                    assert_eq!(runtime.control.parent_number(2, &runtime.global), 1);
                    assert_eq!(runtime.control.parent_number(i32::MAX, &runtime.global), 0);
                    assert_eq!(runtime.control.parent_number(0, &runtime.global), 3);
                    assert_eq!(runtime.control.parent_number(-1, &runtime.global), 3);
                    assert_eq!(
                        runtime.control.frame_environment(0, &runtime.global)?,
                        runtime.global
                    );
                    assert_ne!(execution, runtime.global);
                    for which in [1, 3] {
                        assert_eq!(
                            runtime.control.frame_environment(which, &runtime.global)?,
                            execution
                        );
                    }
                    for which in [2, -1] {
                        assert_eq!(
                            runtime.control.frame_environment(which, &runtime.global)?,
                            wrapper_frame
                        );
                    }
                    assert_eq!(
                        runtime.control.frame_environment(-3, &runtime.global)?,
                        runtime.global
                    );
                    assert_eq!(
                        runtime.control.frame_function(0)?,
                        Sexp::Primitive(r_ust_sexp::PrimitiveId::Quote)
                    );
                    assert_eq!(runtime.control.frame_function(1)?, function);
                    assert_eq!(runtime.control.frame_function(2)?, wrapper);
                    assert_eq!(runtime.control.frame_function(-3)?, Sexp::Nil);
                    for which in [4, -4, i32::MAX, i32::MIN] {
                        assert_eq!(
                            runtime
                                .control
                                .frame_function(which)
                                .map_err(EvalError::unhandled),
                            Err(EvalError::Domain(
                                "not that many frames on the stack".into()
                            ))
                        );
                    }
                    assert_eq!(
                        runtime.control.parent_environment(1, &runtime.global),
                        wrapper_frame
                    );
                    assert_eq!(
                        runtime.control.parent_environment(2, &runtime.global),
                        execution
                    );
                    assert_eq!(
                        runtime
                            .control
                            .parent_environment(i32::MAX, &runtime.global),
                        runtime.global
                    );
                    Ok(EvalFlow::Value(Sexp::Nil))
                })
            })
        })
    })
    .unwrap();
    assert!(runtime.control.contexts.is_empty());
}

#[test]
fn actual_callable_is_retained() {
    let mut source = Frontend::default();
    let actual = source
        .eval_source("x<-17L;function(y=23L)x+y")
        .unwrap()
        .pop()
        .unwrap();
    let lexical = source.runtime().global.clone();
    let alias = actual.clone();
    drop(source);
    let mut runtime = RuntimeState::new(Default::default());
    let caller = runtime.current.clone();
    let observer = closure(&caller, Sexp::Nil);
    host_closure(&mut runtime, &alias, |runtime, body| {
        assert_eq!(runtime.current.parent()?, Some(lexical.clone()));
        assert_eq!(
            runtime
                .current
                .session_root(r_ust_sexp::EnvironmentRole::Global)?,
            lexical
        );
        assert_eq!(runtime.control.contexts.last().unwrap().sysparent, caller);
        assert_eq!(runtime.control.contexts.last().unwrap().callable, actual);
        host_closure(runtime, &observer, |runtime, _| {
            assert_eq!(runtime.control.frame_function(0)?, actual);
            assert_eq!(
                runtime.control.parent_environment(1, &runtime.global),
                caller
            );
            Ok(EvalFlow::Value(Sexp::Nil))
        })?;
        runtime.eval_flow(body)
    })
    .map(|value| assert_eq!(value, EvalFlow::Value(Sexp::Integer(vec![Some(40)]))))
    .unwrap();
    assert!(runtime.control.contexts.is_empty());
    assert_eq!(runtime.current, caller);
}

#[test]
fn selection_uses_active_sysparent_and_saturates_self_parent() {
    let mut runtime = RuntimeState::new(Default::default());
    let selected = runtime.current.child().unwrap();
    let context = ReturnContext {
        execution: selected.clone(),
        sysparent: selected.clone(),
        callable: Sexp::Nil,
        arguments: Vec::new(),
        call: Sexp::Nil,
        generic: false,
    };
    runtime
        .with_return_context(context, |runtime| {
            assert_eq!(runtime.control.selected_depth(), 1);
            assert_eq!(runtime.control.parent_number(i32::MAX, &runtime.global), 1);
            assert_eq!(
                runtime
                    .control
                    .parent_environment(i32::MAX, &runtime.global),
                selected
            );
            Ok(EvalFlow::Value(Sexp::Nil))
        })
        .unwrap();
    assert_eq!(runtime.control.frame_function(0).unwrap(), Sexp::Nil);
    assert_eq!(runtime.control.selected_depth(), 0);
}

#[test]
fn loops_keep_base_sysparent_without_adding_function_depth() {
    let mut runtime = RuntimeState::new(Default::default());
    let function = closure(&runtime.global, Sexp::Nil);
    host_closure(&mut runtime, &function, |runtime, _| {
        runtime.with_loop(|runtime, _| {
            assert_eq!(
                runtime.control.contexts.last().unwrap().sysparent,
                runtime
                    .global
                    .session_root(r_ust_sexp::EnvironmentRole::Base)?
            );
            assert_eq!(runtime.control.selected_depth(), 0);
            host_closure(runtime, &function, |runtime, _| {
                assert_eq!(runtime.control.selected_depth(), 1);
                Ok(EvalFlow::Value(Sexp::Nil))
            })
        })
    })
    .unwrap();
    assert!(runtime.control.contexts.is_empty());
}
