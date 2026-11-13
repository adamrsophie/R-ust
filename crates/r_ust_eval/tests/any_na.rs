use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn any_na_matches_independent_r_values_and_errors() {
    let mut failures = Vec::new();
    let fixture = include_str!("fixtures/any-na-r-4.6.0.tsv");
    assert_eq!(fixture.lines().next(), Some("source\tkind\tresult"));
    assert_eq!(fixture.lines().count(), 91);
    for line in fixture.lines().skip(1) {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3, "fixture row: {line}");
        let result = Frontend::default().eval_source(fields[0]);
        let actual = match result {
            Ok(values) => match values.last() {
                Some(Sexp::Logical(values)) if values == &[Some(true)] => ("value", "TRUE".into()),
                Some(Sexp::Logical(values)) if values == &[Some(false)] => {
                    ("value", "FALSE".into())
                }
                other => ("unexpected", format!("{other:?}")),
            },
            Err(
                EvalError::Arity(message) | EvalError::Type(message) | EvalError::Domain(message),
            ) => ("error", message),
            Err(other) => ("error", other.to_string()),
        };
        if actual != (fields[1], fields[2].to_string()) {
            failures.push(format!(
                "{}: expected {:?}, actual {actual:?}",
                fields[0],
                &fields[1..]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn any_na_evaluates_before_validation_and_stops_at_direct_empty_arguments() {
    for (source, printed, expected) in [
        (
            "anyNA(print(1),print(2),print(3))",
            vec![1, 2, 3],
            "anyNA takes 1 or 2 arguments",
        ),
        (
            "anyNA(y=print(1))",
            vec![1],
            "supplied argument name 'y' does not match 'x'",
        ),
        (
            "anyNA(x=print(1),x=print(2))",
            vec![1, 2],
            "formal argument \"x\" matched by multiple actual arguments",
        ),
        ("anyNA(,print(2))", vec![], "argument 1 is empty"),
        ("anyNA(print(1),)", vec![1], "argument 2 is empty"),
    ] {
        let mut frontend = Frontend::default();
        let error = frontend.eval_source(source).unwrap_err();
        assert_eq!(error, EvalError::Arity(expected.into()), "{source}");
        assert_eq!(
            frontend.runtime_mut().take_output(),
            printed
                .iter()
                .map(|value| OutputEvent::Stdout(format!("[1] {value}\n")))
                .collect::<Vec<_>>(),
            "{source}"
        );
    }
}

#[test]
fn any_na_partial_matching_uses_existing_warning_option() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("options(warn=1,warnPartialMatchArgs=TRUE)")
        .unwrap();
    assert_eq!(
        frontend
            .eval_source("anyNA(list(list(NA)),r=TRUE)")
            .unwrap(),
        vec![Sexp::Logical(vec![Some(true)])]
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stderr(
            "Warning: partial argument match of 'r' to 'recursive'\n".into()
        )]
    );
}
