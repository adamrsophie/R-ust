use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;

#[path = "support/run_length.rs"]
mod support;

#[test]
fn run_length_matches_independent_r_storage_values_errors_and_warnings() {
    let mut lines = include_str!("fixtures/run-length-r-4.6.0.tsv").lines();
    assert_eq!(
        lines.next(),
        Some("# R version 4.6.0 (2026-04-24); class rle omitted from canonical representation")
    );
    let rows: Vec<_> = lines.collect();
    assert_eq!(rows.len(), 49);
    for row in rows {
        let fields: [&str; 4] = row
            .split('\t')
            .collect::<Vec<_>>()
            .try_into()
            .expect("four fixture fields");
        assert!(!fields[0].is_empty());
        assert!(!fields[2].is_empty());
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .set_option("warn", Sexp::Integer(vec![Some(1)]))
            .unwrap();
        let result = frontend.eval_source(fields[0]);
        match fields[1] {
            "OK" => {
                let values = result.unwrap_or_else(|error| panic!("{}: {error:?}", fields[0]));
                assert_eq!(values.len(), 1, "{}", fields[0]);
                assert_eq!(support::encode(&values[0]), fields[2], "{}", fields[0]);
            }
            "ERR" => assert_eq!(
                support::error_payload(result.expect_err(fields[0])),
                fields[2],
                "{}",
                fields[0]
            ),
            other => panic!("unknown fixture outcome {other}"),
        }
        assert_eq!(
            frontend.runtime_mut().take_output(),
            support::warning_events(&fields[3].replace('|', "\u{1f}")),
            "{}",
            fields[0]
        );
    }
}

#[test]
fn run_length_argument_validation_and_large_count_limit_are_explicit() {
    for (source, expected) in [
        ("rle()", EvalError::MissingArgument("x".into())),
        ("rle(1,2)", EvalError::Arity("unused argument (2)".into())),
        (
            "rle(y=1)",
            EvalError::Arity("unused argument (y = 1)".into()),
        ),
        ("inverse.rle()", EvalError::MissingArgument("x".into())),
        (
            "inverse.rle(x=list(lengths=1L,values=2L),x=list(lengths=1L,values=3L))",
            EvalError::Arity("formal argument \"x\" matched by multiple actual arguments".into()),
        ),
        (
            "inverse.rle(list(lengths=2147483648,values=1))",
            EvalError::NotImplemented("long-vector run lengths"),
        ),
        (
            "inverse.rle(list(lengths=c(2147483647L,1L),values=1:2))",
            EvalError::NotImplemented("long-vector run lengths"),
        ),
    ] {
        assert_eq!(
            Frontend::default().eval_source(source),
            Err(expected),
            "{source}"
        );
    }
}

#[path = "support/wrapper_control.rs"]
mod wrapper_control;

#[test]
fn run_length_preserves_nested_control_and_visibility() {
    wrapper_control::check(
        "run_length",
        &[
            "rle_return",
            "inverse_invisible_return",
            "rle_break",
            "inverse_next",
        ],
    );
}
