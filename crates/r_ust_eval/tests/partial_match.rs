use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn partial_matching_agrees_with_r_values_errors_and_warnings() {
    let fixture = include_str!("fixtures/partial-match-r-4.6.0.tsv");
    assert_eq!(fixture.lines().count(), 66);
    assert_eq!(
        fixture.lines().next(),
        Some("# R version 4.6.0 (2026-04-24)")
    );
    let mut failures = Vec::new();
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 6);
        assert_eq!(fields[0], (index + 1).to_string());
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .set_option("warn", Sexp::Integer(vec![Some(1)]))
            .unwrap();
        let result = frontend.eval_source(fields[1]);
        let actual = match result {
            Ok(values) => match values.last().unwrap() {
                Sexp::Integer(values) => (
                    "integer",
                    if values.is_empty() {
                        "-".into()
                    } else {
                        values
                            .iter()
                            .map(|value| {
                                value.map_or_else(|| "NA".into(), |value| value.to_string())
                            })
                            .collect::<Vec<_>>()
                            .join(",")
                    },
                    "-".into(),
                ),
                other => panic!("unexpected value for {}: {other:?}", fields[1]),
            },
            Err(error) => (
                "error",
                "-".into(),
                match error {
                    EvalError::Type(message)
                    | EvalError::Arity(message)
                    | EvalError::Domain(message) => message,
                    other @ (EvalError::Parse(_)
                    | EvalError::UndefinedSymbol(_)
                    | EvalError::UndefinedFunction(_)
                    | EvalError::Stop { .. }
                    | EvalError::Control(_)
                    | EvalError::Transfer(_)
                    | EvalError::MissingArgument(_)
                    | EvalError::Output(_)
                    | EvalError::Input(_)
                    | EvalError::Quit(_)
                    | EvalError::NotImplemented(_)
                    | EvalError::WithWarnings { .. }) => other.to_string(),
                },
            ),
        };
        let output = frontend.runtime_mut().take_output();
        let expected_output: Vec<_> = if fields[5] == "-" {
            Vec::new()
        } else {
            fields[5]
                .split('|')
                .map(|warning| OutputEvent::Stderr(format!("Warning: {warning}\n")))
                .collect()
        };
        let passed = actual.0 == fields[2]
            && actual.1 == fields[3]
            && actual.2 == fields[4]
            && output == expected_output;
        println!(
            "ROW\t{}\t{}\t{}",
            fields[0],
            if passed { "PASS" } else { "FAIL" },
            fields[1]
        );
        if !passed {
            failures.push(format!(
                "{}: got {actual:?}, output {output:?}; expected {line}",
                fields[0]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
