use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

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

fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn fingerprint(value: &Sexp) -> (String, String) {
    match value {
        Sexp::Integer(values) => (
            "integer".into(),
            format!(
                "integer|{}|{}",
                values.len(),
                values
                    .iter()
                    .map(|v| v.map_or_else(|| "NA".into(), |v| v.to_string()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ),
        Sexp::List(values) => (
            "list".into(),
            format!(
                "list|{}|{}",
                values.len(),
                values
                    .iter()
                    .map(|entry| {
                        let name = entry
                            .name
                            .as_ref()
                            .map_or_else(|| "_".into(), |name| hex(name.as_bytes()));
                        format!("{name}={}", fingerprint(&entry.value).1)
                    })
                    .collect::<Vec<_>>()
                    .join(";")
            ),
        ),
        other => panic!("unexpected fixture value {other:?}"),
    }
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
fn all_closure_matching_rows_match_r_4_6_0() {
    let fixture = include_str!("fixtures/closure-matching-r-4.6.0.tsv");
    assert_eq!(fixture.lines().next(), Some("# R version 4.6.0 (2026-04-24) aarch64-apple-darwin23 LDOUBLE 8 JIT 0 DIRECT_TOP_LEVEL"));
    let mut failures = Vec::new();
    let mut counts = [0; 5];
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 7, "{line}");
        let id: usize = fields[0].parse().unwrap();
        assert_eq!(id, index + 1);
        let expected_error = decode_hex(fields[4]);
        let expected_warnings = decode_hex(fields[5]);
        let expected_output = decode_hex(fields[6]);
        assert!(matches!(fields[2], "list" | "integer" | "error"));
        assert_eq!(fields[2] == "error", !expected_error.is_empty());
        assert_eq!(fields[2] == "error", fields[3] == "-");
        let kind_index = match fields[2] {
            "list" => 0,
            "integer" => 1,
            "error" => 2,
            other => panic!("unexpected fixture type {other}"),
        };
        counts[kind_index] += 1;
        counts[3] += usize::from(!expected_warnings.is_empty());
        counts[4] += usize::from(!expected_output.is_empty());
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
            && error == expected_error
            && warnings == expected_warnings
            && output == expected_output;
        println!(
            "ROW\t{id}\t{}\t{}\t{kind}\t{value}\t{}\t{}\t{}",
            if passed { "PASS" } else { "FAIL" },
            fields[1],
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
    assert_eq!(counts, [736, 6, 467, 40, 9]);
    assert_eq!(fixture.lines().count(), 1210);
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
