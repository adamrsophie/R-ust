use crate::{EvalError, Frontend, OutputEvent, Sexp};

mod fixtures;
mod names;
mod node;
mod quit;
mod sdk;
mod terminal;

pub(super) struct Row {
    pub id: &'static str,
    pub setup: &'static str,
    pub source: &'static str,
    pub binding: &'static str,
    pub replacement: &'static str,
    pub error_message: &'static str,
    pub status: i32,
    pub stdout: &'static str,
    pub stderr: &'static str,
}

#[test]
fn warning_restart_invocation_and_lapply_match_literal_r_references() {
    for row in fixtures::rows() {
        eprintln!("REFERENCE_CASE\t{}", row.id);
        let mut frontend = Frontend::default();
        if !row.binding.is_empty() {
            sdk::install_inventory(&mut frontend, row.binding, row.replacement);
        }
        frontend
            .eval_source("options(warn=1,keep.source=FALSE)")
            .unwrap();
        frontend
            .eval_source(row.setup)
            .unwrap_or_else(|error| panic!("{} setup: {error}", row.id));
        assert!(
            frontend.runtime.take_output().is_empty(),
            "{} setup output",
            row.id
        );
        let result = frontend.eval_source(&format!("{{{}}}", row.source));
        let visible = frontend.runtime.is_visible();
        let (stdout, stderr) = streams(&mut frontend);
        if row.status == 4 {
            assert!(
                matches!(result, Err(EvalError::Quit(ref request)) if request.status == 4 && !request.run_last),
                "{}: {result:?}",
                row.id
            );
            assert_eq!(stdout, row.stdout, "{}", row.id);
            assert_eq!(stderr, row.stderr, "{}", row.id);
        } else if row.status == 1 {
            let error = result.expect_err(row.id);
            terminal::assert_error(&error, &row);
            assert_eq!(stdout, row.stdout, "{}", row.id);
            assert!(
                stderr.is_empty(),
                "{} unexpected SDK warning output: {stderr}",
                row.id
            );
        } else {
            let value = result
                .unwrap_or_else(|error| panic!("{}: {error}", row.id))
                .pop()
                .expect("result");
            let (expected_stdout, observation) = row
                .stdout
                .split_once("D2NODE1\n")
                .expect("literal observation");
            let mut lines = observation.lines();
            assert_eq!(
                visible,
                lines.next() == Some("visible"),
                "{} visibility",
                row.id
            );
            let expected = node::Node::read(&mut lines);
            assert!(lines.next().is_none(), "{} trailing observer data", row.id);
            assert_eq!(node::observe(&value), expected, "{} value", row.id);
            assert_eq!(stdout, expected_stdout, "{} stdout", row.id);
            assert_eq!(stderr, row.stderr, "{} stderr", row.id);
        }
        assert!(
            frontend.runtime.conditions.restarts.is_none(),
            "{} restart leak",
            row.id
        );
        assert!(
            frontend.runtime.conditions.head.is_none(),
            "{} handler leak",
            row.id
        );
        assert!(
            frontend.runtime.control.contexts.is_empty(),
            "{} context leak",
            row.id
        );
    }
}

#[test]
fn raised_warning_handler_errors_recover_on_default_test_stack() -> Result<(), EvalError> {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("options(warnPartialMatchArgs=TRUE);tryCatch(withCallingHandlers(is.vector(m='expression',x=expression(x)),warning=function(w)stop('HANDLER')),error=function(e)conditionMessage(e))")?;
    assert_eq!(
        values.last(),
        Some(&Sexp::String(vec![Some("HANDLER".into())]))
    );
    assert_eq!(streams(&mut frontend), (String::new(), String::new()));
    assert!(frontend.runtime.conditions.head.is_none());
    assert!(frontend.runtime.conditions.restarts.is_none());
    assert!(frontend.runtime.control.contexts.is_empty());
    assert_eq!(
        frontend.eval_source("7L")?,
        vec![Sexp::Integer(vec![Some(7)])]
    );
    Ok(())
}

fn streams(frontend: &mut Frontend) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();
    for event in frontend.runtime.take_output() {
        match event {
            OutputEvent::Stdout(text) => stdout.push_str(&text),
            OutputEvent::Stderr(text) => stderr.push_str(&text),
            _ => panic!("unexpected output event"),
        }
    }
    (stdout, stderr)
}
