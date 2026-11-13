use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn hex(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return "-".into();
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn warning_payload(text: &str) -> String {
    if let Some(payload) = text.strip_prefix("Warning message:\n") {
        return payload.strip_suffix(" \n").unwrap().into();
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
        EvalError::Arity(message) | EvalError::Domain(message) | EvalError::Type(message) => {
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
fn encode_string_matches_r_4_6_0_escaping_width_and_argument_contract() {
    let fixture = include_str!("fixtures/encode-string-r-4.6.0.tsv");
    assert_eq!(
        fixture.lines().next(),
        Some("# R 4.6.0 en_US.UTF-8 encodeString v1")
    );
    assert_eq!(fixture.lines().count(), 61);
    let mut failures = Vec::new();
    for (index, line) in fixture.lines().skip(1).enumerate() {
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 6, "{line}");
        assert_eq!(fields[0].parse::<usize>().unwrap(), index + 1);
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
        let (value, error) = match &result {
            Ok(values) => {
                let Sexp::String(values) = values.last().unwrap() else {
                    panic!("{result:?}");
                };
                let value = if values.is_empty() {
                    "EMPTY".into()
                } else {
                    values
                        .iter()
                        .map(|value| {
                            value
                                .as_ref()
                                .map_or_else(|| "NA".into(), |value| hex(value.as_bytes()))
                        })
                        .collect::<Vec<_>>()
                        .join(",")
                };
                (value, "-".into())
            }
            Err(error) => ("-".into(), hex(condition(error, &mut warnings).as_bytes())),
        };
        let actual = [
            value,
            error,
            hex(warnings.join("\n").as_bytes()),
            hex(&output),
        ];
        let passed = actual
            .iter()
            .map(String::as_str)
            .eq(fields[2..].iter().copied());
        println!(
            "ROW\t{}\t{}\t{}",
            fields[0],
            if passed { "PASS" } else { "FAIL" },
            fields[1]
        );
        if !passed {
            failures.push(format!(
                "row {}: {result:?}; actual {actual:?}; expected {:?}",
                fields[0],
                &fields[2..]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[path = "support/wrapper_control.rs"]
mod wrapper_control;

#[test]
fn encode_string_preserves_nested_control_and_visibility() {
    wrapper_control::check(
        "encode_string",
        &[
            "x_return",
            "x_invisible_return",
            "x_break",
            "x_next",
            "justify_return",
            "justify_invisible_return",
            "justify_break",
            "justify_next",
            "width_return",
            "width_invisible_return",
            "width_break",
            "width_next",
            "quote_return",
            "quote_invisible_return",
            "quote_break",
            "quote_next",
            "na_encode_return",
            "na_encode_invisible_return",
            "na_encode_break",
            "na_encode_next",
            "literal_return",
            "alias_quote_return",
            "default_width_invisible_return",
            "invisible_around_return",
            "success_invisible_formals",
        ],
    );
}
