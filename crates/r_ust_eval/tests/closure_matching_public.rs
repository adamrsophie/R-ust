use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{CallArgument, FunctionParameter, ListElement, Sexp};
use std::sync::atomic::{AtomicUsize, Ordering};

const WARNING: &str = "partial argument match of 'al' to 'alpha'";
static WRITES: AtomicUsize = AtomicUsize::new(0);

fn integer(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}

fn call(function: Sexp, arguments: Vec<CallArgument>) -> Sexp {
    Sexp::Call {
        function: Box::new(function),
        arguments,
    }
}

fn function(parameters: Vec<FunctionParameter>, body: Sexp) -> Sexp {
    Sexp::Function {
        parameters,
        body: Box::new(body),
    }
}

fn list(values: &[(&str, Sexp)]) -> Sexp {
    Sexp::List(
        values
            .iter()
            .map(|(name, value)| ListElement {
                name: Some((*name).into()),
                value: value.clone(),
            })
            .collect(),
    )
}

fn warning_events(enabled: bool, text: String) -> Vec<OutputEvent> {
    enabled
        .then_some(OutputEvent::Stderr(text))
        .into_iter()
        .collect()
}

fn failing_sink(_: OutputEvent) -> std::io::Result<()> {
    WRITES.fetch_add(1, Ordering::SeqCst);
    Err(std::io::Error::other("closed matching output"))
}

#[test]
fn public_matching_paths() {
    let parameters = vec![
        FunctionParameter::required("alpha"),
        FunctionParameter::required("beta"),
    ];
    let body = call(
        Sexp::Symbol("list".into()),
        vec![
            CallArgument::named("alpha", Sexp::Symbol("alpha".into())),
            CallArgument::named("beta", Sexp::Symbol("beta".into())),
        ],
    );
    for tag in ["alpha", "al"] {
        let expression = call(
            function(parameters.clone(), body.clone()),
            vec![
                CallArgument::positional(integer(1)),
                CallArgument::named(tag, integer(2)),
            ],
        );
        assert_eq!(
            Frontend::default().runtime_mut().eval(&expression).unwrap(),
            list(&[("alpha", integer(2)), ("beta", integer(1))]),
            "{tag}"
        );
    }
    let mut frontend = Frontend::default();
    frontend.eval_source("f<-function(alpha=unforced_default,...,omega=7L)list(alpha=alpha,dots=list(...),omega=omega)").unwrap();
    assert_eq!(
        frontend.eval_source("f(alpha=,3L,om=5L,omega=)").unwrap(),
        vec![list(&[
            ("alpha", integer(3)),
            ("dots", list(&[("om", integer(5))])),
            ("omega", integer(7))
        ])]
    );
    frontend
        .eval_source("f<-function(alpha=11L,beta=missing_name)list(m=missing(alpha),a=alpha)")
        .unwrap();
    assert_eq!(
        frontend.eval_source("f(al=)").unwrap(),
        vec![list(&[
            ("m", Sexp::Logical(vec![Some(true)])),
            ("a", integer(11))
        ])]
    );
}

#[test]
fn constructed_duplicate_formals_preserve_matching_precedence() {
    let formals = vec![FunctionParameter::required("alpha"); 2];
    for (count, message) in [
        (1, "argument 1 matches multiple formal arguments"),
        (
            2,
            "formal argument \"alpha\" matched by multiple actual arguments",
        ),
    ] {
        let arguments = vec![CallArgument::named("alpha", integer(1)); count];
        let expression = call(
            function(formals.clone(), Sexp::Symbol("unreached_body".into())),
            arguments,
        );
        assert_eq!(
            Frontend::default().runtime_mut().eval(&expression),
            Err(EvalError::Arity(message.into()))
        );
    }
}

#[test]
fn frontend_visibility_and_printing_evaluate_matching_once() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("f<-function(alpha){cat('BODY');invisible(alpha)}")
        .unwrap();
    assert!(frontend.eval_source_visible("f(al=3L)").unwrap().is_empty());
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stdout("BODY".into())]
    );
    frontend
        .eval_source("f<-function(alpha){cat('BODY');alpha}")
        .unwrap();
    frontend.eval_source_printed("f(al=5L)").unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![
            OutputEvent::Stdout("BODY".into()),
            OutputEvent::Stdout("[1] 5\n".into())
        ]
    );
}

#[test]
fn runtime_matching_warning_requires_explicit_finish() {
    for enabled in [false, true] {
        for warn in [-1, 0, 1, 2] {
            let mut frontend = Frontend::default();
            let runtime = frontend.runtime_mut();
            runtime
                .set_option("warnPartialMatchArgs", Sexp::Logical(vec![Some(enabled)]))
                .unwrap();
            runtime.set_option("warn", integer(warn)).unwrap();
            let expression = call(
                function(
                    vec![FunctionParameter::required("alpha")],
                    Sexp::Symbol("alpha".into()),
                ),
                vec![CallArgument::named("al", integer(3))],
            );
            let result = runtime.eval(&expression);
            let immediate = format!("Warning: {WARNING}\n");
            assert_eq!(
                runtime.take_output(),
                warning_events(enabled && warn == 1, immediate)
            );
            if enabled && warn == 2 {
                assert_eq!(
                    result,
                    Err(EvalError::Domain(format!(
                        "(converted from warning) {WARNING}"
                    )))
                );
            } else {
                assert_eq!(result.unwrap(), integer(3));
            }
            runtime.finish_parse_warnings(Ok(())).unwrap();
            let deferred = format!("Warning message:\n{WARNING} \n");
            assert_eq!(
                runtime.take_output(),
                warning_events(enabled && warn == 0, deferred)
            );
            runtime.finish_parse_warnings(Ok(())).unwrap();
            assert!(runtime.take_output().is_empty());
        }
    }
}

#[test]
fn matching_failures_restore_runtime() {
    for (source, expected) in [
        (
            "h<-function(){guard<-99L;f<-function(alpha)alpha;f(alpha=1L,alpha=2L)};h()",
            EvalError::Arity(
                "formal argument \"alpha\" matched by multiple actual arguments".into(),
            ),
        ),
        (
            "h<-function(){guard<-99L;f<-function(alpha,alpine)alpha;f(al=1L)};h()",
            EvalError::Arity("argument 1 matches multiple formal arguments".into()),
        ),
        (
            "g<-function()NULL;h<-function(...){guard<-99L;g(...)};h(unknown=missing_name)",
            EvalError::UndefinedSymbol("missing_name".into()),
        ),
    ] {
        let mut frontend = Frontend::default();
        frontend.eval_source("guard<-17L").unwrap();
        assert_eq!(frontend.eval_source(source), Err(expected), "{source}");
        assert_eq!(
            frontend.eval_source("1+1;guard").unwrap(),
            vec![Sexp::Real(vec![2.0]), integer(17)]
        );
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
    for warn in [0, 1] {
        WRITES.store(0, Ordering::SeqCst);
        let mut frontend = Frontend::default();
        frontend
            .eval_source("guard<-17L;f<-function(alpha){guard<-99L;alpha}")
            .unwrap();
        frontend
            .runtime_mut()
            .set_option("warnPartialMatchArgs", Sexp::Logical(vec![Some(true)]))
            .unwrap();
        frontend
            .runtime_mut()
            .set_option("warn", integer(warn))
            .unwrap();
        frontend.runtime_mut().set_output_sink(failing_sink);
        assert_eq!(
            frontend.eval_source("f(al=3L)"),
            Err(EvalError::Output("closed matching output".into()))
        );
        frontend
            .runtime_mut()
            .finish_parse_warnings(Ok(()))
            .unwrap();
        assert_eq!(
            frontend.eval_source("1+1;guard").unwrap(),
            vec![Sexp::Real(vec![2.0]), integer(17)]
        );
        assert_eq!(WRITES.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn nonlocal_matching_transfers_restore_public_runtime() {
    for source in [
        "f<-function(x)x;h<-function(){guard<-99L;f(return(42L));0L};h()",
        "g<-function()NULL;f<-function(...)g(...);h<-function(){guard<-99L;f(unused=return(42L));0L};h()",
        "g<-function()NULL;f<-function(...)g(...);h<-function(){guard<-99L;for(i in 1:2){f(unused=break);missing_name};42L};h()",
        "g<-function()NULL;f<-function(...)g(...);h<-function(){guard<-99L;for(i in 1:2){f(unused=next);missing_name};42L};h()",
    ] {
        let mut frontend = Frontend::default();
        frontend.eval_source("guard<-17L").unwrap();
        assert_eq!(frontend.eval_source_visible(source).unwrap(), vec![integer(42)], "{source}");
        assert_eq!(frontend.eval_source("1+1;guard").unwrap(), vec![Sexp::Real(vec![2.0]), integer(17)]);
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
}
