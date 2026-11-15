use super::*;

#[test]
fn canonical_break_and_next_preserve_live_loop_targets() {
    for (name, is_break) in [("break", true), ("next", false)] {
        let mut runtime = RuntimeState::new(Default::default());
        runtime
            .with_loop(|runtime, id| {
                let expected = if is_break {
                    crate::ControlTransfer::Break(id)
                } else {
                    crate::ControlTransfer::Next(id)
                };
                assert_eq!(
                    runtime.eval_flow(&call(name, vec![])),
                    Err(EvalError::Transfer(expected))
                );
                Ok(EvalFlow::Value(Sexp::Nil))
            })
            .unwrap();
        assert!(runtime.control.contexts.is_empty());
    }
}

#[test]
fn canonical_promise_control_preserves_origin_and_return_visibility() {
    for canonical in [false, true] {
        let mut runtime = RuntimeState::new(Default::default());
        assert_eq!(
            source(
                &mut runtime,
                "f<-function(x)x;n<-0L;for(i in 1:3){f(break);n<-n+1L};n",
                canonical
            ),
            integer(0)
        );
        assert_eq!(
            source(
                &mut runtime,
                "f<-function(x)x;n<-0L;for(i in 1:3){f(next);n<-n+1L};n",
                canonical
            ),
            integer(0)
        );
        assert_eq!(
            source(
                &mut runtime,
                "f<-function(){return(invisible(6L));stop(\"BAD\")};f()",
                canonical
            ),
            integer(6)
        );
        assert!(!runtime.visible);
        assert!(runtime.control.contexts.is_empty());
    }
}

#[test]
fn selected_private_context_passes_canonical_loop_transfers_without_public_eval() {
    let mut runtime = RuntimeState::new(Default::default());
    runtime
        .with_loop(|runtime, id| {
            let entry = crate::dynamic_context::ReturnContext {
                execution: runtime.current.clone(),
                sysparent: runtime.current.clone(),
                callable: Sexp::Nil,
                arguments: vec![],
                call: Sexp::Nil,
                generic: false,
            };
            assert_eq!(
                runtime.eval_selected(entry, &call("break", vec![])),
                Err(EvalError::Transfer(crate::ControlTransfer::Break(id)))
            );
            Ok(EvalFlow::Value(Sexp::Nil))
        })
        .unwrap();
    assert!(runtime.control.contexts.is_empty());
}

#[test]
fn empty_for_initializes_variable_and_assignment_visibility_is_invisible() {
    for canonical in [false, true] {
        let mut runtime = RuntimeState::new(Default::default());
        source(
            &mut runtime,
            "i<-42L;for(i in integer()) stop(\"BAD\")",
            canonical,
        );
        assert_eq!(runtime.lookup("i").unwrap(), Some(Sexp::Nil));
        assert!(!runtime.visible);
        assert_eq!(source(&mut runtime, "x<-7L", canonical), integer(7));
        assert!(!runtime.visible);
    }
}
