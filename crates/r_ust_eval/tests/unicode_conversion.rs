use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;
use std::fmt::Write as _;

fn hex(bytes: &[u8]) -> String {
    let mut result = String::new();
    for byte in bytes {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}

fn decode(text: &str) -> String {
    if text == "-" {
        return String::new();
    }
    assert!(!text.is_empty() && text.len().is_multiple_of(2));
    let bytes = text
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            assert!(pair.iter().all(u8::is_ascii_hexdigit));
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()
        })
        .collect();
    String::from_utf8(bytes).unwrap()
}

fn fingerprint(value: &Sexp) -> (String, String) {
    let (kind, len, data) = match value {
        Sexp::Integer(values) => (
            "integer",
            values.len(),
            values
                .iter()
                .map(|value| value.map_or_else(|| "NA".into(), |value| value.to_string()))
                .collect::<Vec<_>>()
                .join(","),
        ),
        Sexp::String(values) => (
            "character",
            values.len(),
            values
                .iter()
                .map(|value| {
                    value.as_ref().map_or_else(
                        || "NA".into(),
                        |value| format!("x{}", hex(value.as_bytes())),
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
        Sexp::Logical(values) => (
            "logical",
            values.len(),
            values
                .iter()
                .map(|value| match value {
                    Some(true) => "T",
                    Some(false) => "F",
                    None => "NA",
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
        other => panic!("unexpected oracle result {other:?}"),
    };
    (kind.into(), format!("{kind}|{len}|{data}"))
}

fn message(error: EvalError) -> String {
    match error {
        EvalError::Arity(message) | EvalError::Type(message) | EvalError::Domain(message) => {
            message
        }
        other => other.to_string(),
    }
}

#[test]
fn unicode_conversion_matches_independent_r_values_errors_and_warnings() {
    let fixture = include_str!("fixtures/unicode-conversion-r-4.6.0.tsv");
    assert_eq!(
        fixture.lines().next(),
        Some("# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 UNICODE_CONVERSION_V1")
    );
    assert_eq!(fixture.lines().count(), 70);
    let mut failures = Vec::new();
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split("\t").collect();
        assert_eq!(fields.len(), 7, "{line}");
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        assert!(!fields[1].is_empty());
        assert!(matches!(
            fields[2],
            "integer" | "character" | "logical" | "error"
        ));
        let expected_error = decode(fields[4]);
        assert_eq!(fields[2] == "error", !expected_error.is_empty());
        assert_eq!(fields[2] == "error", fields[3] == "-");
        let expected_stderr = decode(fields[5]);
        let expected_stdout = decode(fields[6]);
        let mut frontend = Frontend::default();
        frontend.eval_source("options(warn=1)").unwrap();
        let result = frontend.eval_source(fields[1]);
        let (kind, value, error) = match result {
            Ok(values) => {
                let (kind, value) = fingerprint(values.last().unwrap());
                (kind, value, String::new())
            }
            Err(error) => ("error".into(), "-".into(), message(error)),
        };
        let mut stdout = String::new();
        let mut stderr = String::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => stdout.push_str(&text),
                OutputEvent::Stderr(text) => stderr.push_str(&text),
                OutputEvent::StdoutBytes(bytes) => {
                    stdout.push_str(std::str::from_utf8(&bytes).unwrap())
                }
            }
        }
        let passed = kind == fields[2]
            && value == fields[3]
            && error == expected_error
            && stdout == expected_stdout
            && stderr == expected_stderr;
        println!(
            "ROW\t{}\t{}\t{}\t{kind}\t{value}\t{}\t{}\t{}",
            index + 1,
            if passed { "PASS" } else { "FAIL" },
            fields[1],
            hex(error.as_bytes()),
            hex(stderr.as_bytes()),
            hex(stdout.as_bytes())
        );
        if !passed {
            failures.push(format!("row {}: {kind} {value}, error {error:?}, stderr {stderr:?}, stdout {stdout:?}; expected {line}",index+1));
        }
    }
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
