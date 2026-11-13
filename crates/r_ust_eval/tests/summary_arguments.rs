use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{CallArgument, Sexp};

fn message(error: EvalError) -> String {
    match error {
        EvalError::Arity(text) | EvalError::Domain(text) | EvalError::Type(text) => text,
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn summary_diagnostics_match_independent_r_fixture() {
    let mut failures = Vec::new();
    let mut checked = 0;
    for line in include_str!("fixtures/summary-r-4.6.0.tsv").lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        if fields[5] == "-" {
            continue;
        }
        let bytes: Vec<_> = (0..fields[5].len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&fields[5][i..i + 2], 16).unwrap())
            .collect();
        let expected = String::from_utf8(bytes).unwrap();
        let expected = match expected.as_str() {
            "‘sum’ not meaningful for factors" => "'sum' not meaningful for factors".into(),
            "‘prod’ not meaningful for factors" => "'prod' not meaningful for factors".into(),
            _ => expected,
        };
        let mut frontend = Frontend::default();
        let actual = frontend.eval_source(fields[1]).map_err(message);
        if actual.as_ref().err() != Some(&expected) {
            failures.push(format!(
                "{}: {}: {actual:?}; expected {expected}",
                fields[0], fields[1]
            ));
        }
        checked += 1;
    }
    assert_eq!(checked, 116);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn scalar_flags_and_exact_names_preserve_data_arguments() {
    for name in ["sum", "prod"] {
        for flag in [
            "TRUE",
            "'TRUE'",
            "'True'",
            "'true'",
            "'T'",
            "1L",
            "-1",
            "Inf",
            "factor('a')",
            "matrix(TRUE)",
        ] {
            let mut frontend = Frontend::default();
            assert_eq!(
                frontend
                    .eval_source(&format!("{name}(c(2,NA_real_),na.rm={flag})"))
                    .unwrap(),
                vec![Sexp::Real(vec![2.0])],
                "{name}: {flag}"
            );
        }
        for (args, expected) in [
            ("x=2,na.r=3", if name == "sum" { 5.0 } else { 6.0 }),
            ("na.rm='FALSE',x=2", 2.0),
        ] {
            assert_eq!(
                Frontend::default()
                    .eval_source(&format!("{name}({args})"))
                    .unwrap(),
                vec![Sexp::Real(vec![expected])]
            );
        }
    }
}

#[test]
fn missing_arguments_stop_evaluation_at_the_empty_position_including_dots() {
    for name in ["sum", "prod"] {
        for (args, index, printed) in [
            ("print(9),", 2, true),
            (",print(9)", 1, false),
            ("na.rm=", 1, false),
        ] {
            for source in [
                format!("{name}({args})"),
                format!("f<-function(...) {name}(...);f({args})"),
            ] {
                let mut frontend = Frontend::default();
                assert_eq!(
                    message(frontend.eval_source(&source).unwrap_err()),
                    format!("argument {index} is empty"),
                    "{source}"
                );
                assert_eq!(
                    frontend.runtime_mut().take_output(),
                    if printed {
                        vec![OutputEvent::Stdout("[1] 9\n".into())]
                    } else {
                        vec![]
                    },
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn eager_warning_and_output_precede_summary_validation() {
    for name in ["sum", "prod"] {
        let mut frontend = Frontend::default();
        frontend.eval_source("options(warn=1)").unwrap();
        assert_eq!(
            message(
                frontend
                    .eval_source(&format!(
                        "{name}(print(9),as.double('bad'),na.rm=list(TRUE))"
                    ))
                    .unwrap_err()
            ),
            "NA in coercion to boolean"
        );
        assert_eq!(
            frontend.runtime_mut().take_output(),
            vec![
                OutputEvent::Stdout("[1] 9\n".into()),
                OutputEvent::Stderr("Warning: NAs introduced by coercion\n".into())
            ]
        );
    }
}

#[test]
fn public_calls_validate_empty_matrix_storage_and_drop_dimensions() {
    for name in ["sum", "prod"] {
        for (data, expected) in [
            (
                Sexp::Integer(vec![Some(2), Some(3)]),
                Ok(if name == "sum" {
                    Sexp::Integer(vec![Some(5)])
                } else {
                    Sexp::Real(vec![6.0])
                }),
            ),
            (
                Sexp::String(vec![]),
                Err("invalid 'type' (character) of argument"),
            ),
            (Sexp::List(vec![]), Err("invalid 'type' (list) of argument")),
        ] {
            let call = Sexp::Call {
                function: Box::new(Sexp::Symbol(name.into())),
                arguments: vec![CallArgument::positional(Sexp::Matrix {
                    data: Box::new(data),
                    nrow: 1,
                    ncol: 2,
                })],
            };
            let actual = Frontend::default()
                .runtime_mut()
                .eval(&call)
                .map_err(message);
            assert_eq!(actual, expected.map_err(String::from));
        }
    }
}

#[test]
fn only_the_first_data_argument_dispatches_factor_summaries() {
    for constructor in ["factor", "ordered"] {
        for (name, prefix, expected) in [
            ("sum", "NULL", Sexp::Integer(vec![Some(3)])),
            ("sum", "1L", Sexp::Integer(vec![Some(4)])),
            ("sum", "1", Sexp::Real(vec![4.0])),
            ("prod", "NULL", Sexp::Real(vec![2.0])),
        ] {
            let source = format!("{name}({prefix},{constructor}(c('a','b')))");
            assert_eq!(
                Frontend::default().eval_source(&source).unwrap(),
                vec![expected],
                "{source}"
            );
        }
    }
}
