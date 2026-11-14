use r_ust_eval::{EvalError, Frontend, OutputEvent, RuntimeState};
use r_ust_parse::Parser;
use r_ust_sexp::{CallArgument, ClosureEnvironment, InternalOperation, Sexp};

#[test]
fn stop_helper_message_preserves_list_coercion_and_append_lf() {
    for (source, expected) in [
        (".makeMessage('x',appendLF=TRUE)", "x\n"),
        ("as.character(list(c(1L,2L)))", "1:2"),
        ("as.character(list(c(2L,1L)))", "2:1"),
        ("as.character(list(c(1L,3L)))", "c(1, 3)"),
        ("as.character(list(c(1L,NA_integer_)))", "c(1, NA)"),
        ("as.character(list(c(1,NA_real_)))", "c(1, NA)"),
        ("as.character(list(factor('a')))", "1"),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(source).unwrap(),
            [Sexp::String(vec![Some(expected.into())])],
            "{source}"
        );
    }
}

#[test]
fn stop_helper_dots_count_is_lazy_and_selection_forces_only_selected() {
    for (source, expected, output) in [
        ("(function(...)...length())(cat('NO'),stop('NO'))", 2, ""),
        ("...length<-1L;(function(...)...length())(1L)", 1, ""),
        (
            "...length<-function()9L;(function(...)...length())(1L)",
            9,
            "",
        ),
        ("(function(...)..2)(cat('NO'),{cat('YES');2L})", 2, "YES"),
        ("(function(){(function(...)..1)(return(1L));99L})()", 1, ""),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(source).unwrap().last(),
            Some(&Sexp::Integer(vec![Some(expected)])),
            "{source}"
        );
        let expected_output = if output.is_empty() {
            vec![]
        } else {
            vec![OutputEvent::Stdout(output.into())]
        };
        assert_eq!(
            frontend.runtime_mut().take_output(),
            expected_output,
            "{source}"
        );
    }
}

#[test]
fn stop_helper_dots_errors_preserve_inner_missing_arguments() {
    for (source, expected) in [
        (
            "...length()",
            "incorrect context: the current call has no '...' to look in",
        ),
        ("..1", "..1 used in an incorrect context, no ... to look in"),
        (
            "(function(...)..1)()",
            "the ... list contains fewer than 1 element",
        ),
        (
            "(function(...)..2)(1L)",
            "the ... list contains fewer than 2 elements",
        ),
        (
            "(function(...)..1)(x=)",
            "argument \"..1\" is missing, with no default",
        ),
        (
            "(function(...)..1)((function(x)x)())",
            "argument \"x\" is missing, with no default",
        ),
        (
            "f<-function(x)g(x);g<-function(...)..1;f()",
            "argument \"x\" is missing, with no default",
        ),
    ] {
        let error = Frontend::default().eval_source(source).unwrap_err();
        let message = match error {
            EvalError::Domain(message) => message,
            error => error.to_string(),
        };
        assert_eq!(message, expected, "{source}");
    }
}

#[test]
fn stop_helper_inherits_uses_represented_classes_and_validates_which() {
    for (source, expected) in [
        (
            "inherits(1L,c('integer','numeric'),TRUE)",
            vec![Some(1), Some(0)],
        ),
        (
            "inherits(matrix(1:4,2),c('matrix','array','integer'),TRUE)",
            vec![Some(1), Some(2), Some(0)],
        ),
        (
            "inherits(factor(c('a','b'),ordered=TRUE),c('ordered','factor'),TRUE)",
            vec![Some(1), Some(2)],
        ),
    ] {
        assert_eq!(
            Frontend::default().eval_source(source).unwrap(),
            [Sexp::Integer(expected)],
            "{source}"
        );
    }
    assert_eq!(
        Frontend::default().eval_source("inherits(1,'numeric',NA)"),
        Err(EvalError::Domain("NA in coercion to boolean".into()))
    );
}

fn internal(operation: InternalOperation, arguments: &[&str]) -> Sexp {
    Sexp::InternalCall {
        operation,
        arguments: arguments
            .iter()
            .map(|source| {
                CallArgument::positional(Parser.parse_all(source).unwrap().expressions.remove(0))
            })
            .collect(),
    }
}

#[test]
fn base_internal_evaluates_arguments_before_arity_and_preserves_empty_errors() {
    let mut runtime = RuntimeState::default();
    assert_eq!(
        runtime.eval(&internal(InternalOperation::Stop, &["cat('A')"])),
        Err(EvalError::Arity(
            "1 argument passed to .Internal(stop) which requires 2".into()
        ))
    );
    assert_eq!(runtime.take_output(), [OutputEvent::Stdout("A".into())]);
    let call = Sexp::InternalCall {
        operation: InternalOperation::Stop,
        arguments: vec![
            CallArgument::positional(Sexp::MissingArg(String::new())),
            CallArgument::positional(Sexp::String(vec![Some("x".into())])),
        ],
    };
    assert_eq!(
        runtime.eval(&call),
        Err(EvalError::Arity("argument 1 is empty".into()))
    );
    assert_eq!(
        runtime.eval(&internal(InternalOperation::Stop, &["FALSE", "12L"])),
        Err(EvalError::Stop {
            message: "12".into(),
            call_requested: false
        })
    );
}

#[test]
fn base_internal_gettext_validates_trim_and_preserves_disabled_domains() {
    for (domain, trim, expected) in [
        (
            "NULL",
            "NA",
            Err(EvalError::Domain("NA in coercion to boolean".into())),
        ),
        (
            "NULL",
            "c(TRUE,FALSE)",
            Err(EvalError::Domain(
                "'length = 2' in coercion to 'logical(1)'".into(),
            )),
        ),
        ("''", "NA", Ok(Sexp::String(vec![Some("x".into())]))),
        ("NA", "NA", Ok(Sexp::String(vec![Some("x".into())]))),
    ] {
        assert_eq!(
            RuntimeState::default().eval(&internal(
                InternalOperation::Gettext,
                &[domain, "'x'", trim]
            )),
            expected
        );
    }
    assert_eq!(
        RuntimeState::default().eval(&internal(InternalOperation::Gettext, &["NULL", "'x'"])),
        Ok(Sexp::String(vec![Some("x".into())]))
    );
}

#[test]
fn base_internal_bootstrap_has_r_formals_and_condition_operations_stay_explicit() {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("stop").unwrap();
    let [Sexp::Closure {
        parameters,
        environment: ClosureEnvironment::Captured(namespace),
        ..
    }] = values.as_slice()
    else {
        panic!("base stop must be a closure")
    };
    assert_eq!(
        frontend.runtime().get(".BaseNamespaceEnv").unwrap(),
        Some(Sexp::Environment(namespace.clone()))
    );
    assert_ne!(
        RuntimeState::default().get(".BaseNamespaceEnv").unwrap(),
        Some(Sexp::Environment(namespace.clone()))
    );
    assert_eq!(
        parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>(),
        ["...", "call.", "domain"]
    );
    assert_eq!(parameters[1].default, Some(Sexp::Logical(vec![Some(true)])));
    assert_eq!(parameters[2].default, Some(Sexp::Nil));
    for (operation, arguments) in [
        (
            InternalOperation::SignalCondition,
            vec!["NULL", "'x'", "NULL"],
        ),
        (InternalOperation::DefaultStop, vec!["'x'", "NULL"]),
    ] {
        assert_eq!(
            RuntimeState::default().eval(&internal(operation, &arguments)),
            Err(EvalError::NotImplemented("condition-object signaling"))
        );
    }
}
