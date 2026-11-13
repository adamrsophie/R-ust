use std::collections::BTreeSet;

use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn decode(text: &str) -> String {
    if text == "-" {
        return String::new();
    }
    assert!(!text.is_empty() && text.len().is_multiple_of(2));
    String::from_utf8(
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                assert!(pair.iter().all(u8::is_ascii_hexdigit));
                u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()
            })
            .collect(),
    )
    .unwrap()
}

fn warning_payload(text: &str) -> String {
    if let Some(payload) = text.strip_prefix("Warning message:\n") {
        return payload.strip_suffix(" \n").unwrap().to_owned();
    }
    if let Some(payload) = text.strip_prefix("Warning messages:\n") {
        return payload
            .lines()
            .enumerate()
            .map(|(index, line)| {
                line.strip_prefix(&format!("{}: ", index + 1))
                    .unwrap()
                    .strip_suffix(' ')
                    .unwrap()
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    text.lines()
        .map(|line| line.strip_prefix("Warning: ").unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn condition(error: &EvalError, warnings: &mut Vec<String>) -> String {
    match error {
        EvalError::Arity(message)
        | EvalError::Domain(message)
        | EvalError::Type(message)
        | EvalError::Stop { message, .. } => message.clone(),
        EvalError::WithWarnings {
            error,
            warnings: text,
        } => {
            warnings.push(warning_payload(text));
            condition(error, warnings)
        }
        other => other.to_string(),
    }
}

#[test]
fn bitwise_matches_r_4_6_values_conditions_forcing_and_visibility() {
    let fixture = include_str!("fixtures/bitwise-r-4.6.0.tsv");
    assert_eq!(
        fixture.lines().next(),
        Some("id\tname\tsource\ttype\tvalues\tvisible\terror_hex\twarnings_hex\tstdout_hex")
    );
    assert_eq!(fixture.lines().count(), 303);
    let mut names = BTreeSet::new();
    let mut failures = Vec::new();
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 9, "{line}");
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        assert!(names.insert(fields[1]));
        assert!(matches!(fields[3], "integer" | "error"));
        assert_eq!(fields[3] == "error", fields[6] != "-");
        assert!(matches!(fields[5], "TRUE" | "FALSE" | "-"));
        assert_eq!(fields[3] == "error", fields[5] == "-");
        let mut frontend = Frontend::default();
        frontend.eval_source("options(warn=1)").unwrap();
        let result = frontend.eval_source(fields[2]);
        let visible = if result.is_err() {
            "-"
        } else if frontend.runtime().is_visible() {
            "TRUE"
        } else {
            "FALSE"
        };
        let mut warnings = Vec::new();
        let mut output = String::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => output.push_str(&text),
                OutputEvent::StdoutBytes(bytes) => {
                    output.push_str(&String::from_utf8(bytes).unwrap());
                }
                OutputEvent::Stderr(text) => warnings.push(warning_payload(&text)),
            }
        }
        let (kind, value, error) = match &result {
            Ok(values) => match values.last() {
                Some(Sexp::Integer(values)) => {
                    let value = if values.is_empty() {
                        "-".into()
                    } else {
                        values
                            .iter()
                            .map(|value| value.map_or_else(|| "NA".into(), |v| v.to_string()))
                            .collect::<Vec<_>>()
                            .join(",")
                    };
                    ("integer", value, String::new())
                }
                other => ("unexpected", format!("{other:?}"), String::new()),
            },
            Err(error) => ("error", "-".into(), condition(error, &mut warnings)),
        };
        let passed = kind == fields[3]
            && value == fields[4]
            && visible == fields[5]
            && error == decode(fields[6])
            && warnings.join("\n") == decode(fields[7])
            && output == decode(fields[8]);
        println!(
            "ROW\t{}\t{}\t{}",
            fields[0],
            fields[1],
            if passed { "PASS" } else { "FAIL" }
        );
        if !passed {
            failures.push(format!(
                "{line}\nactual: {result:?}; visible: {visible}; warnings: {warnings:?}; output: {output:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
