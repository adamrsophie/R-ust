use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn scalar_consumers_convert_only_first_element() {
    for warn in [-1, 0, 1, 2, 3] {
        for first in ["7", "0x7", "7.9"] {
            let mut frontend = Frontend::default();
            frontend
                .eval_source(&format!(
                    "options(warn={warn});options(digits=c('{first}','bad'));getOption('digits')"
                ))
                .unwrap();
            assert_eq!(
                frontend.runtime().get_option("digits"),
                Some(&Sexp::Integer(vec![Some(7)]))
            );
            assert!(frontend.runtime_mut().take_output().is_empty());
        }
        for (first, status) in [("0", 0), ("0x7", 7), ("7.9", 7)] {
            let mut frontend = Frontend::default();
            frontend
                .eval_source(&format!("options(warn={warn})"))
                .unwrap();
            let error = frontend
                .eval_source(&format!(
                    "q(status=c('{first}','bad'),save='no',runLast=FALSE)"
                ))
                .unwrap_err();
            let EvalError::Quit(request) = error else {
                panic!("expected quit, got {error}")
            };
            assert_eq!(request.status, status);
            assert!(!request.run_last);
            frontend
                .runtime_mut()
                .finish_parse_warnings(Ok(()))
                .unwrap();
            assert!(frontend.runtime_mut().take_output().is_empty());
        }
    }
}

#[test]
fn option_warning_precedes_validation_and_reader_is_silent() {
    for (input, warning) in [
        ("'bad'", "NAs introduced by coercion"),
        (
            "'2147483648'",
            "NAs introduced by coercion to integer range",
        ),
    ] {
        for warn in [-1, 0, 1, 2, 3] {
            let mut frontend = Frontend::default();
            frontend
                .eval_source(&format!("options(digits=9,warn={warn})"))
                .unwrap();
            let error = frontend
                .eval_source(&format!("options(digits=c({input},'7'))"))
                .unwrap_err();
            assert_eq!(
                frontend.runtime().get_option("digits"),
                Some(&Sexp::Integer(vec![Some(9)]))
            );
            if warn >= 2 {
                assert_eq!(
                    error,
                    EvalError::Domain(format!("(converted from warning) {warning}"))
                );
            } else {
                assert!(error
                    .to_string()
                    .contains("invalid 'digits' parameter, allowed 1...22"));
                assert_eq!(error.to_string().contains(warning), warn == 0);
            }
            let expected = if warn == 1 {
                vec![OutputEvent::Stderr(format!("Warning: {warning}\n"))]
            } else {
                vec![]
            };
            assert_eq!(frontend.runtime_mut().take_output(), expected);
        }
    }

    let mut frontend = Frontend::default();
    frontend
        .eval_source("options(warning.length=c('100','bad'),nwarnings=c('2','bad'),warn=1)")
        .unwrap();
    assert_eq!(
        frontend.runtime().get_option("warning.length"),
        Some(&Sexp::String(vec![Some("100".into()), Some("bad".into())]))
    );
    frontend.eval_source("as.double('bad')").unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stderr(
            "Warning: NAs introduced by coercion\n".into()
        )]
    );
}

#[test]
fn missing_scalar_values_fail_validation_without_coercion_warnings() {
    for warn in [-1, 0, 1, 2, 3] {
        for input in ["''", "' '", "'NaN'", "NA_real_", "NA_integer_"] {
            let mut frontend = Frontend::default();
            frontend
                .eval_source(&format!("options(digits=9,warn={warn})"))
                .unwrap();
            let error = frontend
                .eval_source(&format!("options(digits={input})"))
                .unwrap_err();
            assert_eq!(
                error,
                EvalError::Domain("invalid 'digits' parameter, allowed 1...22".into()),
                "{input}; warn={warn}"
            );
            assert_eq!(
                frontend.runtime().get_option("digits"),
                Some(&Sexp::Integer(vec![Some(9)]))
            );
            assert!(frontend.runtime_mut().take_output().is_empty());
        }
    }
}

#[test]
fn quit_bad_first_value_promotes_before_invalid_status_warning() {
    for (input, warning) in [
        ("bad", "NAs introduced by coercion"),
        ("2147483648", "NAs introduced by coercion to integer range"),
    ] {
        for warn in [2, 3] {
            let mut frontend = Frontend::default();
            frontend
                .eval_source(&format!("options(warn={warn})"))
                .unwrap();
            assert_eq!(
                frontend
                    .eval_source(&format!(
                        "q(status=c('{input}','0'),save='no',runLast=FALSE)"
                    ))
                    .unwrap_err(),
                EvalError::Domain(format!("(converted from warning) {warning}"))
            );
            assert!(frontend.runtime_mut().take_output().is_empty());
        }
    }
}

#[test]
fn quit_missing_status_falls_back_after_coercion_warning_delivery() {
    for warn in [-1, 0, 1, 2, 3] {
        for (arguments, warnings) in [
            (
                "status='bad',runLast=FALSE",
                vec!["NAs introduced by coercion", "invalid 'status', 0 assumed"],
            ),
            (
                "status='2147483648',runLast=FALSE",
                vec![
                    "NAs introduced by coercion to integer range",
                    "invalid 'status', 0 assumed",
                ],
            ),
            (
                "status='',runLast=FALSE",
                vec!["invalid 'status', 0 assumed"],
            ),
            (
                "status=NA_integer_,runLast=FALSE",
                vec!["invalid 'status', 0 assumed"],
            ),
            (
                "status=0,runLast=NA",
                vec!["invalid 'runLast', FALSE assumed"],
            ),
        ] {
            let mut frontend = Frontend::default();
            frontend
                .eval_source(&format!("options(warn={warn})"))
                .unwrap();
            let error = frontend
                .eval_source(&format!("q(save='no',{arguments})"))
                .unwrap_err();
            if warn >= 2 {
                assert_eq!(
                    error,
                    EvalError::Domain(format!("(converted from warning) {}", warnings[0]))
                );
                assert!(frontend.runtime_mut().take_output().is_empty());
                continue;
            }
            let EvalError::Quit(request) = error else {
                panic!("expected quit, got {error}")
            };
            assert_eq!(request.status, 0);
            assert!(!request.run_last);
            frontend
                .runtime_mut()
                .finish_parse_warnings(Ok(()))
                .unwrap();
            let expected = match warn {
                -1 => vec![],
                1 => warnings
                    .iter()
                    .map(|warning| OutputEvent::Stderr(format!("Warning: {warning}\n")))
                    .collect(),
                _ => vec![OutputEvent::Stderr(if warnings.len() == 1 {
                    format!("Warning message:\n{} \n", warnings[0])
                } else {
                    format!(
                        "Warning messages:\n1: {} \n2: {} \n",
                        warnings[0], warnings[1]
                    )
                })],
            };
            assert_eq!(
                frontend.runtime_mut().take_output(),
                expected,
                "{arguments}; warn={warn}"
            );
        }
    }
    let mut frontend = Frontend::default();
    frontend.eval_source("options(warn=2)").unwrap();
    let error = frontend
        .eval_source("q(status='NaN',save='no',runLast=FALSE)")
        .unwrap_err();
    let EvalError::Quit(request) = error else {
        panic!("expected quit, got {error}")
    };
    assert_eq!(request.status, 0);
    assert!(frontend.runtime_mut().take_output().is_empty());
}
