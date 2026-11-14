use r_ust_eval::{format_sexp, EvalError, Frontend, OutputEvent, RuntimeState};
use r_ust_sexp::{CallArgument, ClosureEnvironment, InternalOperation, ListElement, Sexp};

#[test]
fn closure_display_includes_actual_body_and_environment() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source_printed(
            "function(x=1L,...) { x + 1L }; identity; function(x=stop('UNFORCED'))stop('UNFORCED'); sequence",
        )
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("function (x = 1L, ...) \n{\n    x + 1L\n}\n".into()),
            OutputEvent::Stdout("function (x) \nx\n<environment: namespace:base>\n".into()),
            OutputEvent::Stdout("function (x = stop(\"UNFORCED\")) \nstop(\"UNFORCED\")\n".into()),
            OutputEvent::Stdout(
                "function (nvec, ...) \nUseMethod(\"sequence\")\n<environment: namespace:base>\n"
                    .into()
            ),
        ]
    );
    let closure = frontend
        .eval_source("f<-function(){function(x)x+1L};f()")
        .unwrap()
        .pop()
        .unwrap();
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(frame),
        ..
    } = &closure
    else {
        panic!("expected captured closure");
    };
    assert_eq!(
        format_sexp(&closure).unwrap(),
        format!(
            "function (x) \nx + 1L\n<environment: {}>",
            frame.display_identity().unwrap()
        )
    );
}

#[test]
fn nested_closure_display_preserves_prefixes() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source_printed("list(a=function(x)x+1L,b=list(function(y=2L){y}))")
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout(
            "$a\nfunction (x) \nx + 1L\n\n$b\n$b[[1]]\nfunction (y = 2L) \n{\n    y\n}\n\n\n"
                .into()
        )]
    );
}

#[test]
fn optimized_sdk_body_fails_explicitly() {
    let closure = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::InternalCall {
            operation: InternalOperation::Formals,
            arguments: vec![],
        }),
        environment: ClosureEnvironment::Base,
    };
    let expected = EvalError::NotImplemented("function body source representation");
    assert_eq!(format_sexp(&closure), Err(expected.clone()));
    let mut runtime = RuntimeState::default();
    assert_eq!(
        runtime.eval(&Sexp::InternalCall {
            operation: InternalOperation::Body,
            arguments: vec![CallArgument::positional(closure.clone())],
        }),
        Err(expected)
    );
    assert!(runtime.take_output().is_empty());
    let container = Sexp::List(vec![
        ListElement {
            name: None,
            value: Sexp::Integer(vec![Some(7)]),
        },
        ListElement {
            name: None,
            value: closure,
        },
    ]);
    assert_eq!(
        runtime.eval(&Sexp::Call {
            function: Box::new(Sexp::Symbol("print".into())),
            arguments: vec![CallArgument::positional(container)],
        }),
        Err(EvalError::NotImplemented(
            "function body source representation"
        ))
    );
    assert!(runtime.take_output().is_empty());
    let mut frontend = Frontend::default();
    for source in ["rle", "encodeString", "rowsum.default"] {
        assert_eq!(
            frontend.eval_source_printed(source),
            Err(EvalError::NotImplemented(
                "function body source representation"
            )),
            "{source}"
        );
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
}
