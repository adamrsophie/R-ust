use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;
use std::fmt::Write as _;

fn raw_hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}

fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    raw_hex(bytes)
}

fn decode_hex(text: &str) -> Vec<u8> {
    if text == "-" {
        return Vec::new();
    }
    assert!(!text.is_empty() && text.len().is_multiple_of(2));
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            assert!(pair.iter().all(u8::is_ascii_hexdigit));
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap()
        })
        .collect()
}

fn fingerprint(value: &Sexp) -> (String, String) {
    let Sexp::Integer(values) = value else {
        panic!("unexpected sequence storage {value:?}");
    };
    let data = values
        .iter()
        .map(|value| value.map_or_else(|| "NA".into(), |value| value.to_string()))
        .collect::<Vec<_>>()
        .join(",");
    ("integer".into(), format!("integer|{}|{data}", values.len()))
}

fn warning_payload(text: &str) -> String {
    if let Some(payload) = text.strip_prefix("Warning message:\n") {
        return payload.strip_suffix(" \n").unwrap().to_string();
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
        EvalError::Arity(message) | EvalError::Domain(message) | EvalError::Control(message) => {
            message.clone()
        }
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
fn sequence_matches_independent_r_4_6_0_fixture() {
    let fixture = include_str!("fixtures/sequence-r-4.6.0.tsv");
    assert_eq!(
        fixture.lines().next(),
        Some("# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 JIT 0 FRESH_PROCESS_PER_ROW")
    );
    let mut failures = Vec::new();
    let mut compared = 0;
    let mut limited = 0;
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split("\t").collect();
        assert_eq!(fields.len(), 7, "{line}");
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        if index + 1 == 69 {
            limited += 1;
            println!(
                "ROW\t69\tLIMITED\ttryCatch unavailable; warning state checked at host boundary"
            );
            continue;
        }
        compared += 1;
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[1]);
        let mut warnings = Vec::new();
        let mut output = Vec::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => output.extend(text.as_bytes()),
                OutputEvent::StdoutBytes(bytes) => output.extend(bytes),
                OutputEvent::Stderr(text) => warnings.push(warning_payload(&text)),
            }
        }
        let (kind, value, error) = match &result {
            Ok(values) => {
                let (kind, value) = fingerprint(values.last().unwrap());
                (kind, value, Vec::new())
            }
            Err(error) => (
                "error".into(),
                "-".into(),
                condition(error, &mut warnings).into_bytes(),
            ),
        };
        let warnings = warnings.join("\n").into_bytes();
        let passed = kind == fields[2]
            && value == fields[3]
            && error == decode_hex(fields[4])
            && warnings == decode_hex(fields[5])
            && output == decode_hex(fields[6]);
        println!(
            "ROW\t{}\t{}\t{}\t{kind}\t{value}\t{}\t{}\t{}",
            fields[0],
            if passed { "PASS" } else { "FAIL" },
            fields[1],
            hex(&error),
            hex(&warnings),
            hex(&output)
        );
        if !passed {
            failures.push(format!(
                "row {}: {result:?}; expected {line}; warnings {}; stdout {}",
                fields[0],
                hex(&warnings),
                hex(&output)
            ));
        }
    }
    assert_eq!(fixture.lines().count(), 91);
    assert_eq!((compared, limited), (89, 1));
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn sequence_warning_promotion_leaves_flag_unset_for_same_runtime_recovery() {
    let fixture = include_str!("fixtures/sequence-r-4.6.0.tsv");
    let fields: Vec<_> = fixture.lines().nth(69).unwrap().split('\t').collect();
    let mut frontend = Frontend::default();
    let promoted = frontend
        .eval_source("options(warn=2);sequence(1,from=c(3,4))")
        .unwrap_err();
    let mut warnings = Vec::new();
    assert_eq!(
        condition(&promoted, &mut warnings).as_bytes(),
        decode_hex(fields[6])
    );
    assert!(warnings.is_empty());
    assert!(frontend.runtime_mut().take_output().is_empty());
    let recovered = frontend
        .eval_source("options(warn=1);sequence(1,from=c(5,6))")
        .unwrap();
    assert_eq!(fingerprint(recovered.last().unwrap()).1, fields[3]);
    let output = frontend.runtime_mut().take_output();
    let [OutputEvent::Stderr(warning)] = output.as_slice() else {
        panic!("expected one recovery warning: {output:?}");
    };
    assert_eq!(warning_payload(warning).as_bytes(), decode_hex(fields[5]));
    println!(
        "HOST_BOUNDARY\tPASS\twarn=2 promotion leaves flag unset; recovery warns once\t{}",
        fields[3]
    );
}

#[path = "support/wrapper_control.rs"]
mod wrapper_control;

#[test]
fn sequence_preserves_nested_control_and_visibility() {
    wrapper_control::check(
        "sequence",
        &[
            "generic_return",
            "default_from_invisible_return",
            "from_break",
            "by_next",
        ],
    );
}
