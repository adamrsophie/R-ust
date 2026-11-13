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
fn strtoi_matches_r_4_6_integer_parsing_and_wrapper_contract() {
    let fixture = include_str!("fixtures/strtoi-r-4.6.0.tsv");
    assert_eq!(
        fixture.lines().next(),
        Some("# R version 4.6.0 (2026-04-24) STRTOI UTF-8 en_US.UTF-8")
    );
    assert_eq!(fixture.lines().count(), 124);
    let mut failures = Vec::new();
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 7, "{line}");
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
        assert!(matches!(fields[2], "integer" | "error"));
        assert_eq!(fields[2] == "error", fields[4] != "-");
        let mut frontend = Frontend::default();
        frontend.eval_source("options(warn=1)").unwrap();
        let result = frontend.eval_source(fields[1]);
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
            Ok(values) => {
                let Some(Sexp::Integer(values)) = values.last() else {
                    panic!("{}: unexpected result {result:?}", fields[1]);
                };
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
            Err(error) => ("error", "-".into(), condition(error, &mut warnings)),
        };
        let passed = kind == fields[2]
            && value == fields[3]
            && error == decode(fields[4])
            && warnings.join("\n") == decode(fields[5])
            && output == decode(fields[6]);
        println!(
            "ROW\t{}\t{}\t{}",
            fields[0],
            if passed { "PASS" } else { "FAIL" },
            fields[1]
        );
        if !passed {
            failures.push(format!(
                "{line}\nactual: {result:?}; warnings: {warnings:?}; output: {output:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
