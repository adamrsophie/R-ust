use super::{format_language, LanguagePrintError};
use crate::{format_sexp, CallArgument, EvalError, Frontend, ListElement, OutputEvent, Sexp};

fn call(value: Sexp) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol("f".into())),
        arguments: vec![CallArgument::positional(value)],
    }
}

#[test]
fn checked_display_propagates_nested_language_failure() {
    let invalid = call(Sexp::Break);
    let element = ListElement {
        name: None,
        value: invalid.clone(),
    };
    for value in [
        Sexp::Vector(vec![Sexp::Nil, invalid.clone()]),
        Sexp::List(vec![element.clone()]),
        Sexp::Dots(vec![element.clone()]),
        Sexp::Matrix {
            data: Box::new(Sexp::List(vec![element.clone()])),
            nrow: 1,
            ncol: 1,
        },
        Sexp::DataFrame {
            columns: vec![ListElement {
                name: Some("x".into()),
                value: Sexp::List(vec![element]),
            }],
            nrow: 1,
        },
    ] {
        assert_eq!(
            format_sexp(&value),
            Err(EvalError::NotImplemented("language printer node"))
        );
    }
}

#[test]
fn containers_keep_their_framing_around_complete_language_text() {
    let value = call(Sexp::Symbol("x".into()));
    let element = ListElement {
        name: None,
        value: value.clone(),
    };
    for (value, expected) in [
        (Sexp::Vector(vec![value]), "f(x)"),
        (Sexp::List(vec![element.clone()]), "[[1]]\nf(x)\n"),
        (Sexp::Dots(vec![element.clone()]), "[[1]]\nf(x)\n"),
        (
            Sexp::Matrix {
                data: Box::new(Sexp::List(vec![element.clone()])),
                nrow: 1,
                ncol: 1,
            },
            "      [,1]\n[1,] f(x)",
        ),
        (
            Sexp::DataFrame {
                columns: vec![ListElement {
                    name: Some("x".into()),
                    value: Sexp::List(vec![element]),
                }],
                nrow: 1,
            },
            "  x\n1 f(x)",
        ),
    ] {
        assert_eq!(format_sexp(&value).unwrap(), expected);
    }
}

#[test]
fn frontend_and_print_emit_complete_language_and_preserve_visibility() -> Result<(), EvalError> {
    let value = call(Sexp::Symbol("x".into()));
    let mut frontend = Frontend::default();
    frontend
        .runtime_mut()
        .current
        .insert("value".into(), &value)?;
    frontend.eval_source_printed("value;print(value);print(value,quote=FALSE);invisible(value)")?;
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![
            OutputEvent::Stdout("f(x)\n".into()),
            OutputEvent::Stdout("f(x)\n".into()),
            OutputEvent::Stdout("f(x)\n".into()),
        ]
    );
    assert!(!frontend.runtime().is_visible());
    Ok(())
}

#[test]
fn rejected_print_preserves_prior_effects_without_partial_display() -> Result<(), EvalError> {
    for source in [
        "cat('before');value;cat('after')",
        "cat('before');print(value);cat('after')",
    ] {
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .current
            .insert("value".into(), &call(Sexp::Break))?;
        assert_eq!(
            frontend.eval_source_printed(source),
            Err(EvalError::NotImplemented("language printer node"))
        );
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![OutputEvent::Stdout("before".into())]
        );
    }
    Ok(())
}

#[test]
fn rejected_print_retains_parse_warnings_and_output_errors() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    frontend
        .runtime_mut()
        .current
        .insert("value".into(), &call(Sexp::Break))?;
    let error = frontend
        .eval_source_printed("print(value,quote=1.1L)")
        .unwrap_err();
    match error {
        EvalError::WithWarnings { error, warnings } => {
            assert_eq!(*error, EvalError::NotImplemented("language printer node"));
            assert_eq!(
                warnings,
                "Warning message:\ninteger literal 1.1L contains decimal; using numeric value \n"
            );
        }
        error => panic!("missing deferred warning: {error:?}"),
    }
    fn fail(_: OutputEvent) -> std::io::Result<()> {
        Err(std::io::Error::other("sink failure"))
    }
    let mut frontend = Frontend::default();
    frontend
        .runtime_mut()
        .current
        .insert("value".into(), &call(Sexp::Nil))?;
    frontend.runtime_mut().set_output_sink(fail);
    assert_eq!(
        frontend.eval_source_printed("value"),
        Err(EvalError::Output("sink failure".into()))
    );
    Ok(())
}

#[test]
fn language_print_rejects_promise_without_forcing_or_reading_it() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    let expression = Sexp::Symbol("FORCED".into());
    let state = r_ust_sexp::PromiseState::new(
        &expression,
        &frontend.runtime().current,
        r_ust_sexp::EvaluationOrigin::Global,
        true,
    )?;
    let value = call(Sexp::Promise {
        state: state.clone(),
    });
    assert_eq!(
        format_language(&value),
        Err(LanguagePrintError::UnsupportedNode)
    );
    assert_eq!(state.expression()?, expression);
    assert!(frontend.runtime_mut().take_output().is_empty());
    Ok(())
}
