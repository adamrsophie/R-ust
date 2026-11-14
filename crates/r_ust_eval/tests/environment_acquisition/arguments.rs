use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn invalid_eager_arguments_preserve_effects_and_exact_errors() {
    for (source, error, effects) in [
        (
            "globalenv({cat('A');1L},{cat('B');2L})",
            "2 arguments passed to 'globalenv' which requires 0",
            vec!["A", "B"],
        ),
        (
            "baseenv(1L)",
            "1 argument passed to 'baseenv' which requires 0",
            vec![],
        ),
        (
            "emptyenv(1L)",
            "1 argument passed to 'emptyenv' which requires 0",
            vec![],
        ),
        ("globalenv(,{cat('BAD');2L})", "argument 1 is empty", vec![]),
        (
            "is.environment()",
            "0 arguments passed to 'is.environment' which requires 1",
            vec![],
        ),
        ("is.environment(x=)", "argument 1 is empty", vec![]),
        ("is.environment(,)", "argument 1 is empty", vec![]),
        (
            "is.environment({cat('A');1L},{cat('B');2L})",
            "2 arguments passed to 'is.environment' which requires 1",
            vec!["A", "B"],
        ),
        (
            "is.environment(env={cat('A');globalenv()})",
            "supplied argument name 'env' does not match 'x'",
            vec!["A"],
        ),
        (
            "is.environment(e=globalenv())",
            "supplied argument name 'e' does not match 'x'",
            vec![],
        ),
        (
            ".Internal(parent.env())",
            "0 arguments passed to .Internal(parent.env) which requires 1",
            vec![],
        ),
        (
            ".Internal(environmentName(globalenv(),{cat('A');1L}))",
            "2 arguments passed to .Internal(environmentName) which requires 1",
            vec!["A"],
        ),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(source),
            Err(EvalError::Arity(error.into())),
            "{source}"
        );
        assert_eq!(
            frontend.runtime_mut().take_output(),
            effects
                .into_iter()
                .map(|s| OutputEvent::Stdout(s.into()))
                .collect::<Vec<_>>(),
            "{source}"
        );
        assert_eq!(
            frontend.eval_source("is.environment(globalenv())").unwrap(),
            [Sexp::Logical(vec![Some(true)])]
        );
    }
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source("is.environment(env=stop('INNER'))"),
        Err(EvalError::Stop {
            message: "INNER".into(),
            call_requested: true
        })
    );
}

#[test]
fn invalid_closure_arguments_are_matched_before_forcing() {
    let mut frontend = Frontend::default();
    for (source, error) in [
        ("parent.env()", EvalError::MissingArgument("env".into())),
        (
            "environmentName()",
            EvalError::MissingArgument("env".into()),
        ),
        (
            "f<-function(x)environmentName(x);f()",
            EvalError::MissingArgument("x".into()),
        ),
        (
            "parent.env(1L)",
            EvalError::Type("argument is not an environment".into()),
        ),
        (
            "parent.env(emptyenv())",
            EvalError::Domain("the empty environment has no parent".into()),
        ),
        (
            "parent.env(env=stop('BAD'),env=stop('BAD'))",
            EvalError::Arity("formal argument \"env\" matched by multiple actual arguments".into()),
        ),
        (
            "parent.env(stop('BAD'),unused=stop('BAD'))",
            EvalError::Arity("unused argument (unused = stop(\"BAD\"))".into()),
        ),
        (
            "environmentName(x=stop('BAD'))",
            EvalError::Arity("unused argument (x = stop(\"BAD\"))".into()),
        ),
    ] {
        assert_eq!(frontend.eval_source(source), Err(error), "{source}");
        assert_eq!(frontend.runtime_mut().take_output(), []);
    }
    frontend
        .eval_source("options(warn=1,warnPartialMatchArgs=TRUE)")
        .unwrap();
    assert_eq!(
        frontend
            .eval_source("environmentName(e=baseenv())")
            .unwrap(),
        [Sexp::String(vec![Some("base".into())])]
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stderr(
            "Warning: partial argument match of 'e' to 'env'\n".into()
        )]
    );
}

#[test]
fn predicate_forces_once_and_apply_uses_the_same_primitive() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend
            .eval_source(concat!(
                "f<-function(x)c(is.environment(x),is.environment(x)); ",
                "f({cat('ONCE');globalenv()}); ",
                "lapply(list(globalenv(),1L),is.environment)"
            ))
            .unwrap()
            .last(),
        Some(&Sexp::List(vec![
            r_ust_sexp::ListElement {
                name: None,
                value: Sexp::Logical(vec![Some(true)])
            },
            r_ust_sexp::ListElement {
                name: None,
                value: Sexp::Logical(vec![Some(false)])
            },
        ]))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("ONCE".into())]
    );
}
