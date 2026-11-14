use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{
    AssignmentOperator, CallArgument, ClosureEnvironment, Environment, EvaluationOrigin, Force,
    ListElement, PromiseState, Sexp,
};

fn frontend() -> Frontend {
    let mut frontend = Frontend::default();
    // Extraction computes the same missing leaf as R's quote(expr=), without a quote primitive.
    frontend
        .runtime_mut()
        .eval(&Sexp::Assignment {
            target: "payload".into(),
            value: Box::new(Sexp::List(vec![ListElement {
                name: None,
                value: Sexp::MissingValue,
            }])),
            operator: AssignmentOperator::Left,
        })
        .unwrap();
    frontend
}

#[test]
fn raw_bindings_remain_stored_and_forward_as_missing() {
    let mut frontend = frontend();
    frontend.eval_source("x <- payload[[1]]").unwrap();
    assert_eq!(
        frontend.runtime().get("x").unwrap(),
        Some(Sexp::MissingValue)
    );
    assert_eq!(
        frontend.eval_source("(function(y) missing(y))(x)").unwrap(),
        vec![Sexp::Logical(vec![Some(true)])]
    );
}

#[test]
fn explicit_sdk_missing_selects_defaults_and_keeps_dots_diagnostics() {
    for (arguments, expected) in [
        (vec![CallArgument::positional(Sexp::MissingValue)], true),
        (vec![CallArgument::named("x", Sexp::MissingValue)], true),
        (
            vec![
                CallArgument::named("x", Sexp::MissingValue),
                CallArgument::positional(Sexp::Integer(vec![Some(7)])),
            ],
            false,
        ),
    ] {
        let mut frontend = frontend();
        frontend
            .eval_source("f <- function(x=99L) c(missing(x), x==99L)")
            .unwrap();
        assert_eq!(
            frontend
                .runtime_mut()
                .eval(&Sexp::Call {
                    function: Box::new(Sexp::Symbol("f".into())),
                    arguments,
                })
                .unwrap(),
            Sexp::Logical(vec![Some(expected), Some(expected)])
        );
    }
    let mut frontend = frontend();
    frontend.eval_source("f <- function(...) ..1").unwrap();
    assert_eq!(
        frontend.runtime_mut().eval(&Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![CallArgument::positional(Sexp::MissingValue)],
        }),
        Err(EvalError::MissingArgument("..1".into()))
    );
}

#[test]
fn missing_runtime_effects_and_diagnostics_match_independent_r_captures() {
    for row in include_str!("fixtures/missing-value-runtime-r-4.6.0.tsv").lines() {
        let fields: [&str; 5] = row.split('\t').collect::<Vec<_>>().try_into().unwrap();
        let mut frontend = frontend();
        let source = fields[1].replace("quote(expr=)", "payload[[1]]");
        let result = frontend.eval_source(&source);
        match fields[2] {
            "0" => {
                result.unwrap_or_else(|error| panic!("{}: {error:?}", fields[0]));
            }
            "1" => {
                let EvalError::MissingArgument(name) = result.unwrap_err() else {
                    panic!("{}", fields[0]);
                };
                let diagnostic = format!("argument \"{name}\" is missing, with no default");
                assert!(
                    fields[4].contains(&hex(diagnostic.as_bytes())),
                    "{}",
                    fields[0]
                );
            }
            status => panic!("unexpected R status {status}"),
        }
        let mut stdout = Vec::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => stdout.extend(text.bytes()),
                OutputEvent::StdoutBytes(bytes) => stdout.extend(bytes),
                OutputEvent::Stderr(text) => panic!("unexpected stderr: {text}"),
            }
        }
        assert_eq!(hex(&stdout), fields[3], "{}", fields[0]);
    }
}

fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        "-".into()
    } else {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

#[test]
fn raw_sdk_dots_reject_lookup_before_forcing() {
    let environment = Environment::global();
    environment
        .insert(
            "...".into(),
            &Sexp::Dots(vec![ListElement {
                name: None,
                value: Sexp::MissingValue,
            }]),
        )
        .unwrap();
    let closure = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Symbol("..1".into())),
        environment: ClosureEnvironment::Captured(environment),
    };
    assert_eq!(
        frontend().runtime_mut().eval(&Sexp::Call {
            function: Box::new(closure),
            arguments: vec![],
        }),
        Err(EvalError::MissingArgument("..1".into()))
    );
}

#[test]
fn pending_and_cached_missing_promises_survive_export_and_origin_drop() {
    for cached in [false, true] {
        let mut frontend = frontend();
        let expression = Sexp::Extract {
            value: Box::new(Sexp::List(vec![ListElement {
                name: None,
                value: Sexp::MissingValue,
            }])),
            index: Box::new(Sexp::Integer(vec![Some(1)])),
        };
        let environment = Environment::global();
        let state =
            PromiseState::new(&expression, &environment, EvaluationOrigin::Global, true).unwrap();
        let promise = Sexp::Promise {
            state: state.clone(),
        };
        if cached {
            assert_eq!(
                frontend.runtime_mut().eval(&promise).unwrap(),
                Sexp::MissingValue
            );
        }
        environment.insert("value".into(), &promise).unwrap();
        let exported = environment.get("value").unwrap().unwrap();
        drop(promise);
        drop(environment);
        drop(frontend);
        let mut runtime = r_ust_eval::RuntimeState::default();
        assert_eq!(runtime.eval(&exported).unwrap(), Sexp::MissingValue);
        assert_eq!(runtime.eval(&exported).unwrap(), Sexp::MissingValue);
        assert!(state.supplied().unwrap());
        assert!(state.environment().unwrap().is_none());
        assert!(matches!(
            state.begin_force().unwrap(),
            Force::Cached(Sexp::MissingValue)
        ));
    }
}
