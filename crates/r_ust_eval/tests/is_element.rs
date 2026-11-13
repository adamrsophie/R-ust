use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn decode(text: &str) -> String {
    if text == "-" {
        return String::new();
    }
    assert!(!text.is_empty() && text.len().is_multiple_of(2));
    assert!(text.bytes().all(|byte| byte.is_ascii_hexdigit()));
    String::from_utf8(
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect(),
    )
    .unwrap()
}

fn warning_payload(text: &str) -> String {
    text.lines()
        .map(|line| line.strip_prefix("Warning: ").unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn represented_is_element_calls_match_r_4_6_0() {
    check_fixture(include_str!("fixtures/is-element-r-4.6.0.tsv"), [37, 3]);
}

#[test]
fn shared_match_coercion_matches_r_4_6_0() {
    check_fixture(include_str!("fixtures/match-coercion-r-4.6.0.tsv"), [20, 0]);
}

fn check_fixture(fixture: &str, expected_counts: [usize; 2]) {
    let mut lines = fixture.lines();
    assert_eq!(lines.next(), Some("# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"));
    let mut failures = Vec::new();
    let mut counts = [0; 2];
    for (index, line) in lines.enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 8, "{line}");
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        assert!(matches!(fields[3], "logical" | "integer" | "error"));
        assert_eq!(fields[3] == "error", fields[4] == "-");
        let expected_error = decode(fields[5]);
        assert_eq!(fields[3] == "error", !expected_error.is_empty());
        let expected_warnings = decode(fields[6]);
        let expected_output = decode(fields[7]);
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[2]);
        let mut warnings = Vec::new();
        let mut output = String::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => output.push_str(&text),
                OutputEvent::StdoutBytes(bytes) => {
                    output.push_str(std::str::from_utf8(&bytes).unwrap());
                }
                OutputEvent::Stderr(text) => warnings.push(warning_payload(&text)),
            }
        }
        let (kind, value, error) = match &result {
            Ok(values) => match values.last().unwrap() {
                Sexp::Integer(values) => (
                    "integer",
                    format!(
                        "integer|{}|{}",
                        values.len(),
                        values
                            .iter()
                            .map(|value| value
                                .map_or_else(|| "NA".into(), |value| value.to_string()))
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    String::new(),
                ),
                Sexp::Logical(values) => (
                    "logical",
                    format!(
                        "logical|{}|{}",
                        values.len(),
                        values
                            .iter()
                            .map(|value| match value {
                                Some(true) => "T",
                                Some(false) => "F",
                                None => "NA",
                            })
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                    String::new(),
                ),
                other => ("other", format!("{other:?}"), String::new()),
            },
            Err(error) => (
                "error",
                "-".into(),
                match error {
                    EvalError::Arity(message)
                    | EvalError::Domain(message)
                    | EvalError::Control(message)
                    | EvalError::Type(message)
                    | EvalError::Stop { message, .. } => message.clone(),
                    other => other.to_string(),
                },
            ),
        };
        let warnings = warnings.join("\n");
        let passed = kind == fields[3]
            && value == fields[4]
            && error == expected_error
            && warnings == expected_warnings
            && output == expected_output;
        let limitation = match fields[1] {
            "list" | "list_scalars" => Some("existing match list support"),
            "object" => Some("custom attributes and S3 support"),
            _ => None,
        };
        if let Some(reason) = limitation {
            counts[1] += 1;
            println!("ROW\t{}\tLIMIT\t{reason}\t{result:?}", fields[0]);
        } else {
            counts[0] += 1;
            println!(
                "ROW\t{}\t{}\t{}\t{kind}\t{value}\t{error:?}\t{warnings:?}\t{output:?}",
                fields[0],
                if passed { "PASS" } else { "FAIL" },
                fields[1]
            );
            if !passed {
                failures.push(format!(
                    "row {}: {result:?}; expected {line}; warnings={warnings:?}; stdout={output:?}",
                    fields[0]
                ));
            }
        }
    }
    assert_eq!(counts, expected_counts);
    assert_eq!(
        fixture.lines().count(),
        1 + expected_counts.iter().sum::<usize>()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
