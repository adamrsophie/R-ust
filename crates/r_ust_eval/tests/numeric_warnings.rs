use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

const WARNING: &str = "integer literal 1.5L contains decimal; using numeric value";

static WRITES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn failing_sink(_: OutputEvent) -> std::io::Result<()> {
    WRITES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Err(std::io::Error::other("closed warning output"))
}

#[test]
fn warn_controls_timing_suppression_and_promotion() {
    for warn in [-1, 0, 1, 2] {
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .set_option("warn", Sexp::Integer(vec![Some(warn)]))
            .unwrap();
        let result = frontend.eval_source_printed("1.5L");
        let value = OutputEvent::Stdout("[1] 1.5\n".into());
        let expected = match warn {
            -1 => vec![value],
            0 => vec![
                value,
                OutputEvent::Stderr(format!("Warning message:\n{WARNING} \n")),
            ],
            1 => vec![OutputEvent::Stderr(format!("Warning: {WARNING}\n")), value],
            _ => Vec::new(),
        };
        assert_eq!(
            frontend.runtime_mut().take_output(),
            expected,
            "warn={warn}"
        );
        if warn == 2 {
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("(converted from warning)"));
        } else {
            result.unwrap();
        }
    }
}

#[test]
fn deferred_warnings_survive_option_changes_and_accompany_errors() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source_printed("{options(warn=-1);1.5L}")
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![
            OutputEvent::Stdout("[1] 1.5\n".into()),
            OutputEvent::Stderr(format!("Warning message:\n{WARNING} \n")),
        ]
    );
    let mut frontend = Frontend::default();
    let error = frontend
        .eval_source_printed("{1.5L;missingName}")
        .unwrap_err();
    assert!(error
        .to_string()
        .contains(&format!("In addition: Warning message:\n{WARNING}")));
    assert!(frontend.runtime_mut().take_output().is_empty());
    let mut frontend = Frontend::default();
    assert!(matches!(
        frontend.eval_source_printed("{1.5L;q('no')}"),
        Err(EvalError::Quit(_))
    ));
    assert!(frontend.runtime_mut().take_output().is_empty());
    frontend
        .runtime_mut()
        .finish_parse_warnings(Ok(()))
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stderr(format!(
            "Warning message:\n{WARNING} \n"
        ))]
    );
}

#[test]
fn all_frontend_entry_points_process_parser_warnings() {
    let mut frontend = Frontend::default();
    frontend.eval_source("1.5L;2.5L").unwrap();
    assert_eq!(frontend.runtime_mut().take_output(), vec![OutputEvent::Stderr(format!(
        "Warning messages:\n1: {WARNING} \n2: integer literal 2.5L contains decimal; using numeric value \n"
    ))]);
    let mut frontend = Frontend::default();
    assert!(frontend
        .eval_source_visible("invisible(1.5L)")
        .unwrap()
        .is_empty());
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stderr(format!(
            "Warning message:\n{WARNING} \n"
        ))]
    );
    assert!(Frontend::default().eval_source_printed("1.5L+@").is_err());
    let mut frontend = Frontend::default();
    assert!(frontend.eval_source_printed("1.5L+").is_err());
    assert!(frontend.runtime_mut().take_output().is_empty());
    let error = Frontend::default().eval_source("1.5L+@").unwrap_err();
    assert!(error.to_string().contains("In addition: Warning message:"));
}

#[test]
fn output_failure_discards_deferred_warnings_without_retrying_the_sink() {
    for level in [0, 1] {
        WRITES.store(0, std::sync::atomic::Ordering::SeqCst);
        let mut frontend = Frontend::default();
        frontend
            .runtime_mut()
            .set_option("warn", Sexp::Integer(vec![Some(level)]))
            .unwrap();
        frontend.runtime_mut().set_output_sink(failing_sink);
        assert!(matches!(
            frontend.eval_source_printed("1.5L"),
            Err(EvalError::Output(_))
        ));
        assert_eq!(WRITES.load(std::sync::atomic::Ordering::SeqCst), 1);
        frontend
            .runtime_mut()
            .finish_parse_warnings(Ok(()))
            .unwrap();
        assert_eq!(WRITES.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}

#[test]
fn warning_length_uses_option_coercion_and_error_header_capacity() {
    let source = format!("0.{}L", "0".repeat(120));
    let mut frontend = Frontend::default();
    frontend
        .runtime_mut()
        .set_option("warning.length", Sexp::Real(vec![100.0]))
        .unwrap();
    frontend.eval_source_printed(&source).unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output().last(),
        Some(&OutputEvent::Stderr(format!(
            "Warning message:\ninteger literal 0.{} [... truncated] \n",
            "0".repeat(82)
        )))
    );
    frontend
        .runtime_mut()
        .set_option("warn", Sexp::Integer(vec![Some(2)]))
        .unwrap();
    assert_eq!(
        frontend.eval_source_printed(&source).unwrap_err(),
        EvalError::Domain(format!(
            "(converted from warning) integer literal 0.{}",
            "0".repeat(50)
        ))
    );
}
