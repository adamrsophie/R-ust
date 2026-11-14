use r_ust_eval::{format_sexp, EvalError, Frontend, OutputEvent};
use r_ust_sexp::{AssignmentOperator, ListElement, Sexp};

const FIXTURE: &str = include_str!("fixtures/operator-primitives-r-4.6.0.tsv");

#[test]
fn arithmetic_primitives_match_independent_r_semantics() {
    check_rows("portable", 246);
}

#[test]
fn sdk_missing_values_match_future_quote_contract_without_quote_support() {
    check_rows("future_quote", 32);
}

fn check_rows(scope: &str, expected_count: usize) {
    let mut count = 0;
    for row in FIXTURE.lines().skip(1) {
        let fields: [&str; 10] = row.split('\t').collect::<Vec<_>>().try_into().unwrap();
        assert!(matches!(fields[1], "portable" | "future_quote"));
        if fields[1] != scope {
            continue;
        }
        count += 1;
        let mut frontend = Frontend::default();
        let source = if scope == "future_quote" {
            // SDK list extraction computes the missing leaf; no public quote implementation is claimed.
            frontend
                .runtime_mut()
                .eval(&Sexp::Assignment {
                    target: "payload".into(),
                    value: Box::new(Sexp::List(vec![ListElement {
                        name: None,
                        value: Sexp::MissingValue,
                    }])),
                    operator: AssignmentOperator::Left,
                })
                .unwrap();
            assert!(fields[2].contains("quote(expr=)"));
            fields[2].replace("quote(expr=)", "payload[[1]]")
        } else {
            fields[2].into()
        };
        let result = frontend.eval_source(&format!("{{{source}}}"));
        let mut warnings = String::new();
        match result {
            Ok(values) => {
                assert_ne!(fields[3], "error", "{}", fields[0]);
                let [value] = values.as_slice() else {
                    panic!("{}: {values:?}", fields[0])
                };
                assert_eq!(fingerprint(value), fields[4], "{}", fields[0]);
                assert_eq!(
                    frontend.runtime().is_visible(),
                    fields[8] == "TRUE",
                    "{}",
                    fields[0]
                );
                if fields[9] != "-" {
                    assert_eq!(
                        format!("{}\n", format_sexp(value).unwrap()),
                        unhex(fields[9]),
                        "{}",
                        fields[0]
                    );
                    for (query, expected) in [
                        ("typeof", "builtin"),
                        ("class", "function"),
                        ("mode", "function"),
                    ] {
                        assert_eq!(
                            frontend.eval_source(&format!("{query}({source})")).unwrap(),
                            vec![Sexp::String(vec![Some(expected.into())])]
                        );
                    }
                    assert_eq!(
                        frontend
                            .eval_source(&format!("is.function({source})"))
                            .unwrap(),
                        vec![Sexp::Logical(vec![Some(true)])]
                    );
                    assert_eq!(
                        frontend.eval_source(&format!("length({source})")).unwrap(),
                        vec![Sexp::Integer(vec![Some(1)])]
                    );
                }
            }
            Err(error) => {
                assert_eq!(fields[3], "error", "{}: {error:?}", fields[0]);
                let (payload, attached) = condition(error);
                assert_eq!(payload, unhex(fields[5]), "{}", fields[0]);
                warnings.push_str(&attached);
            }
        }
        let mut stdout = Vec::new();
        for event in frontend.runtime_mut().take_output() {
            match event {
                OutputEvent::Stdout(text) => stdout.extend(text.bytes()),
                OutputEvent::StdoutBytes(bytes) => stdout.extend(bytes),
                OutputEvent::Stderr(text) => warnings.push_str(&text),
            }
        }
        assert_eq!(stdout, unhex(fields[7]).into_bytes(), "{}", fields[0]);
        let expected = unhex(fields[6]);
        let messages: Vec<_> = expected.lines().collect();
        let expected = match messages.as_slice() {
            [] => String::new(),
            [message] => format!("Warning message:\n{message} \n"),
            _ => format!(
                "Warning messages:\n{}",
                messages
                    .iter()
                    .enumerate()
                    .map(|(i, message)| format!("{}: {message} \n", i + 1))
                    .collect::<String>()
            ),
        };
        assert_eq!(warnings, expected, "{}", fields[0]);
    }
    assert_eq!(count, expected_count);
}

fn condition(error: EvalError) -> (String, String) {
    match error {
        EvalError::WithWarnings { error, warnings } => {
            let (message, nested) = condition(*error);
            assert!(nested.is_empty());
            (message, warnings)
        }
        EvalError::Type(message)
        | EvalError::Arity(message)
        | EvalError::Domain(message)
        | EvalError::Stop { message, .. } => (message, String::new()),
        EvalError::MissingArgument(_) => (error.to_string(), String::new()),
        other => panic!("unrepresented condition {other:?}"),
    }
}

fn unhex(value: &str) -> String {
    if value == "-" {
        return String::new();
    }
    assert_eq!(value.len() % 2, 0);
    String::from_utf8(
        (0..value.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
            .collect(),
    )
    .unwrap()
}

fn fingerprint(value: &Sexp) -> String {
    let (storage, values) = match value {
        Sexp::Primitive(id) => return format!("builtin:.Primitive(\"{}\")", id.name()),
        Sexp::Integer(values) => (
            "integer",
            values
                .iter()
                .map(|v| v.map_or("NA".into(), |v| v.to_string()))
                .collect::<Vec<_>>(),
        ),
        Sexp::Logical(values) => (
            "logical",
            values
                .iter()
                .map(|v| {
                    match v {
                        Some(true) => "T",
                        Some(false) => "F",
                        None => "NA",
                    }
                    .into()
                })
                .collect(),
        ),
        Sexp::Real(values) => (
            "double",
            values
                .iter()
                .map(|v| format!("{:016x}", v.to_bits()))
                .collect(),
        ),
        other => panic!("unrepresented value {other:?}"),
    };
    format!("{storage}|{}|{}", values.len(), values.join(","))
}

#[test]
fn arithmetic_handles_work_through_apply_and_dots() {
    for (source, expected) in [
        ("lapply(list(7L,9L), `-`, 2L)[[2]]", 7),
        ("(function(...) `-`(...))(e2=7L,e1=2L)", 5),
        ("`-` <- function(e1) 99L; -stop('unused')", 99),
    ] {
        assert_eq!(
            Frontend::default().eval_source(source).unwrap().last(),
            Some(&Sexp::Integer(vec![Some(expected)]))
        );
    }
}

#[test]
fn warning_policy_escalates_shared_arithmetic_before_later_effects() {
    for (source, message) in [
        ("2147483647L+1L", "NAs produced by integer overflow"),
        ("`+`(2147483647L,1L)", "NAs produced by integer overflow"),
        (
            "1:3 + 1:2",
            "longer object length is not a multiple of shorter object length",
        ),
    ] {
        let mut frontend = Frontend::default();
        let error = frontend
            .eval_source(&format!("options(warn=2); {source}; cat('AFTER')"))
            .unwrap_err();
        assert_eq!(
            condition(error).0,
            format!("(converted from warning) {message}")
        );
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
}
