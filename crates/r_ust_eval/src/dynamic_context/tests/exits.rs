use super::*;

fn fail_output(_: crate::OutputEvent) -> std::io::Result<()> {
    Err(std::io::Error::other("closed output"))
}

fn fail_input() -> std::io::Result<String> {
    Err(std::io::Error::other("closed input"))
}

#[test]
fn dynamic_stack_restores_after_every_exit() {
    for source in [
        "42L",
        "missing_name",
        "return(42L)",
        "break",
        "next",
        "quit(status=7)",
        "cat('x')",
        "readline()",
    ] {
        let mut runtime = RuntimeState::new(Default::default());
        let original = snapshot(&runtime);
        runtime.set_session_context(vec![], true);
        runtime.set_input_reader(fail_input);
        if source == "cat('x')" {
            runtime.set_output_sink(fail_output);
        }
        let function = closure(&runtime.global, Sexp::Nil);
        let parsed = expression(source);
        let result = runtime.with_loop(|runtime, loop_id| {
            let before = snapshot(runtime);
            let execution = runtime.current.clone();
            let result = host_closure(runtime, &function, |runtime, _| {
                let active = snapshot(runtime);
                let result = runtime.eval_selected(entry(runtime, execution), &parsed);
                assert_eq!(
                    snapshot(runtime),
                    active,
                    "{source}: selected context leaked"
                );
                result
            });
            assert_eq!(
                snapshot(runtime),
                before,
                "{source}: leaked activation or roots"
            );
            let expected = match source {
                "42L" | "return(42L)" => Ok(EvalFlow::Value(Sexp::Integer(vec![Some(42)]))),
                "missing_name" => Err(EvalError::Signaled(Box::new(EvalError::UndefinedSymbol(
                    "missing_name".into(),
                )))),
                "break" => Err(EvalError::Transfer(ControlTransfer::Break(loop_id))),
                "next" => Err(EvalError::Transfer(ControlTransfer::Next(loop_id))),
                "quit(status=7)" => Err(EvalError::Quit(crate::QuitRequest {
                    save: crate::SaveAction::Default,
                    status: 7,
                    run_last: true,
                })),
                "cat('x')" => Err(EvalError::Output("closed output".into())),
                _ => Err(EvalError::Input("closed input".into())),
            };
            assert_eq!(result, expected, "{source}");
            result
        });
        assert_eq!(snapshot(&runtime), original, "{source}: {result:?}");
        assert!(
            runtime.control.contexts.is_empty(),
            "{source}: active roots survived"
        );
        assert_eq!(
            runtime.eval(&Sexp::Integer(vec![Some(43)])).unwrap(),
            Sexp::Integer(vec![Some(43)])
        );
        println!("RESTORED {source}: {result:?}; active contexts=0");
    }
}

#[test]
fn default_promise_return_restores_activation() {
    let mut runtime = RuntimeState::new(Default::default());
    let original = snapshot(&runtime);
    let function = Sexp::Closure {
        parameters: vec![crate::FunctionParameter {
            name: "x".into(),
            default: Some(expression("return(19L)")),
        }],
        body: Box::new(Sexp::Symbol("x".into())),
        environment: ClosureEnvironment::Captured(runtime.global.clone()),
    };
    assert_eq!(
        runtime.call_value(function, &[], &Sexp::Nil).unwrap(),
        EvalFlow::Value(Sexp::Integer(vec![Some(19)]))
    );
    assert_eq!(snapshot(&runtime), original);
}
