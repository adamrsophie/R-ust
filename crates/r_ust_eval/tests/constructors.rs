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
    let (kind, len, data) = match value {
        Sexp::Nil => return ("NULL".into(), "NULL".into()),
        Sexp::Logical(values) => (
            "logical",
            values.len(),
            values
                .iter()
                .map(|v| match v {
                    Some(true) => "T",
                    Some(false) => "F",
                    None => "NA",
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
        Sexp::Integer(values) => (
            "integer",
            values.len(),
            values
                .iter()
                .map(|v| v.map_or_else(|| "NA".into(), |v| v.to_string()))
                .collect::<Vec<_>>()
                .join(","),
        ),
        Sexp::Real(values) => (
            "double",
            values.len(),
            values
                .iter()
                .map(|v| hex(&v.to_be_bytes()))
                .collect::<Vec<_>>()
                .join(","),
        ),
        Sexp::String(values) => (
            "character",
            values.len(),
            values
                .iter()
                .map(|v| {
                    v.as_ref()
                        .map_or_else(|| "NA".into(), |v| format!("x{}", raw_hex(v.as_bytes())))
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
        Sexp::List(values) => (
            "list",
            values.len(),
            values
                .iter()
                .map(|v| {
                    assert!(v.name.is_none(), "unexpected constructor attribute");
                    fingerprint(&v.value).1
                })
                .collect::<Vec<_>>()
                .join(";"),
        ),
        other => panic!("unexpected fixture value {other:?}"),
    };
    (kind.into(), format!("{kind}|{len}|{data}"))
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
fn constructors_match_independent_r_4_6_0_fixture() {
    let fixture = include_str!("fixtures/constructors-r-4.6.0.tsv");
    assert_eq!(fixture.lines().next(), Some("# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"));
    let mut failures = Vec::new();
    let mut counts = [0; 2];
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 8, "{line}");
        let id: usize = fields[0].parse().unwrap();
        assert_eq!(id, index + 1);
        let expected_error = decode_hex(fields[5]);
        let expected_warnings = decode_hex(fields[6]);
        let expected_output = decode_hex(fields[7]);
        assert_eq!(fields[3] == "error", !expected_error.is_empty());
        assert_eq!(fields[3] == "error", fields[4] == "-");
        assert!(matches!(
            fields[3],
            "NULL"
                | "logical"
                | "integer"
                | "double"
                | "character"
                | "list"
                | "error"
                | "raw"
                | "complex"
                | "expression"
                | "pairlist"
        ));
        match fields[1] {
            "represented" => counts[0] += 1,
            "future-type" => {
                counts[1] += 1;
                println!("ROW\t{id}\tUNREPRESENTED\t{}", fields[2]);
                continue;
            }
            other => panic!("unknown fixture lane {other}"),
        }
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(fields[2]);
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
        let passed = kind == fields[3]
            && value == fields[4]
            && error == expected_error
            && warnings == expected_warnings
            && output == expected_output;
        println!(
            "ROW\t{id}\t{}\t{}\t{kind}\t{value}\t{}\t{}\t{}",
            if passed { "PASS" } else { "FAIL" },
            fields[2],
            hex(&error),
            hex(&warnings),
            hex(&output)
        );
        if !passed {
            failures.push(format!(
                "row {id}: {result:?}; expected {line}; warnings {}; stdout {}",
                hex(&warnings),
                hex(&output)
            ));
        }
    }
    assert_eq!(counts, [628, 20]);
    assert_eq!(fixture.lines().count(), 649);
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
