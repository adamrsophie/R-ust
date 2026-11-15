use super::*;

fn case(id: &str) -> (Conditions, Row) {
    let text = if id.starts_with("body_") {
        include_str!("../../fixtures/dynamic-frame-source-r-4.6.0.tsv")
    } else {
        include_str!("../../fixtures/dynamic-frames-r-4.6.0.tsv")
    };
    let count = if id.starts_with("body_") { 30 } else { 184 };
    let row = reader::read(text, count)
        .unwrap()
        .into_iter()
        .find(|row| row.id == id)
        .unwrap();
    (read().unwrap().remove(id).unwrap(), row)
}

fn wrapped_case() -> (Conditions, Row) {
    let (condition, mut row) = case("new_env_hash_coerce_before_parent_validation");
    row.expression = "new.env(hash='bad',parent=NULL,size='bad')".into();
    row.source = format!("{}\n", row.expression);
    row.stderr = include_str!("../../fixtures/dynamic-frame-wrapped-warning-r-4.6.0.stderr").into();
    (condition, row)
}

fn with_warning(condition: &Conditions, warning: String) -> EvalError {
    EvalError::WithWarnings {
        error: Box::new(EvalError::Domain(condition.error.clone().unwrap())),
        warnings: warning,
    }
}

#[test]
fn frozen_warning_errors_reject_call_wrap_and_whitespace_mutations() {
    let (condition, row) = wrapped_case();
    let expected = condition.warnings(&row).unwrap();
    assert!(condition
        .check_error(&with_warning(&condition, expected.into()), &row)
        .is_ok());
    for (name, mutant) in [
        ("missing warning", String::new()),
        (
            "missing call",
            "Warning message:\nNAs introduced by coercion \n".into(),
        ),
        ("changed call", expected.replace("new.env(", "other(")),
        ("removed wrap", expected.replace(" :\n  ", " : ")),
        ("changed indentation", expected.replace("\n  NAs", "\n NAs")),
        (
            "extra whitespace",
            expected.replace("coercion\n", "coercion \n"),
        ),
        ("missing newline", expected.trim_end_matches('\n').into()),
        ("duplicated warning", format!("{expected}{expected}")),
    ] {
        assert!(
            condition
                .check_error(&with_warning(&condition, mutant), &row)
                .is_err(),
            "accepted {name}"
        );
    }
}

#[test]
fn frozen_warning_effects_preserve_bytes_and_stream_order() {
    let (condition, mut row) = case("new_env_hash_bad_string");
    let mut stdout = b"before\n".to_vec();
    stdout.extend(row.stdout);
    row.stdout = stdout;
    let warning = condition.warnings(&row).unwrap().to_owned();
    assert!(condition
        .check_effects(
            vec![
                OutputEvent::Stdout("before\n".into()),
                OutputEvent::Stderr(warning.clone())
            ],
            &row,
        )
        .is_ok());
    for actual in [
        vec![
            OutputEvent::Stderr(warning.clone()),
            OutputEvent::Stdout("before\n".into()),
        ],
        vec![OutputEvent::Stdout("before\n".into())],
        vec![
            OutputEvent::Stdout("before\n".into()),
            OutputEvent::Stderr(warning.replace("new.env(", "other(")),
        ],
        vec![
            OutputEvent::Stdout("before\n".into()),
            OutputEvent::Stderr("Warning message:\nNAs introduced by coercion \n".into()),
        ],
        vec![
            OutputEvent::Stdout("before\n".into()),
            OutputEvent::Stderr(format!("{warning} ")),
        ],
        vec![OutputEvent::Stdout(format!("before\n{warning}"))],
    ] {
        assert!(condition.check_effects(actual, &row).is_err());
    }
}

#[test]
fn frozen_terminal_warning_reference_fails_closed() {
    for change in [
        "missing delimiter",
        "missing terminal suffix",
        "missing heading",
    ] {
        let (condition, mut row) = wrapped_case();
        row.stderr = match change {
            "missing delimiter" => row.stderr.replace("In addition: ", ""),
            "missing terminal suffix" => row.stderr.replace("Execution halted\n", ""),
            _ => row.stderr.replace("Warning message:\n", ""),
        };
        assert!(condition.warnings(&row).is_err(), "accepted {change}");
    }
}

#[test]
fn converted_warning_does_not_emit_a_deferred_warning() {
    let (condition, row) = case("body_warning_error");
    assert!(condition
        .check_error(&EvalError::Domain(condition.error.clone().unwrap()), &row,)
        .is_ok());
    assert!(condition
        .check_error(
            &with_warning(
                &condition,
                "Warning message:\nIn body(fun) : argument is not a function\n".into()
            ),
            &row,
        )
        .is_err());
    assert!(condition.check_effects(Vec::new(), &row).is_ok());
    assert!(condition
        .check_effects(vec![OutputEvent::Stderr("warning\n".into())], &row)
        .is_err());
}

#[test]
fn frozen_reference_rejects_forged_payload_trailing_line_and_duplicate_block() {
    let (condition, row) = wrapped_case();
    let warning = condition.warnings(&row).unwrap();
    for (name, stderr) in [
        (
            "forged payload",
            row.stderr
                .replace("NAs introduced by coercion", "FORGED PAYLOAD"),
        ),
        (
            "trailing line",
            row.stderr.replace(
                "Execution halted\n",
                "FORGED TRAILING LINE\nExecution halted\n",
            ),
        ),
        (
            "duplicate block",
            row.stderr.replace(
                "Execution halted\n",
                &format!("{warning}Execution halted\n"),
            ),
        ),
    ] {
        let (condition, mut mutant) = wrapped_case();
        mutant.stderr = stderr;
        assert!(condition.warnings(&mutant).is_err(), "accepted {name}");
        assert!(condition
            .check_error(&with_warning(&condition, warning.into()), &mutant)
            .is_err());
    }
}

#[test]
fn reference_payload_validation_preserves_valid_raw_bytes_and_warning_order() {
    for id in [
        "new_env_hash_bad_string",
        "new_env_hash_coerce_before_parent_validation",
        "body_nonfunction",
    ] {
        let (condition, row) = case(id);
        let expected = if condition.error.is_some() {
            row.stderr
                .split_once("\nIn addition: ")
                .unwrap()
                .1
                .strip_suffix("Execution halted\n")
                .unwrap()
        } else {
            row.stderr.as_str()
        };
        assert_eq!(condition.warnings(&row).unwrap(), expected);
    }
    let (condition, row) = wrapped_case();
    assert!(condition
        .warnings(&row)
        .unwrap()
        .contains(" :\n  NAs introduced by coercion\n"));
    let messages = vec!["first\ncontinued".into(), "second".into()];
    let valid = "Warning messages:\n1: first\ncontinued \n2: In f(x = 1) : second\n";
    assert!(validate_reference(valid, &messages).is_ok());
    assert!(validate_reference("Warning message:\nplain \n", &["plain".into()]).is_ok());
    for mutant in [
        valid.replace("2: ", "3: "),
        valid.replace("second\n", "FORGED PAYLOAD\n"),
        format!("{valid}FORGED TRAILING LINE\n"),
        format!("{valid}{valid}"),
    ] {
        assert!(validate_reference(&mutant, &messages).is_err());
    }
    assert!(validate_reference(valid, &["second".into(), "first\ncontinued".into()]).is_err());
}
