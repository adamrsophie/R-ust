use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;
use std::collections::BTreeSet;

#[path = "eval_runtime/fixtures.rs"]
mod fixtures;
#[path = "eval_runtime/observe.rs"]
mod observe;

#[test]
fn all_74_frozen_eval_rows_have_exact_typed_results_effects_and_errors() {
    let rows = fixtures::rows();
    assert_eq!(rows.len(), 74);
    let mut ids = BTreeSet::new();
    let mut counts = (0, 0);
    for row in &rows {
        assert!(ids.insert(row.id), "duplicate {}", row.id);
        let mut frontend = Frontend::default();
        frontend.eval_source(fixtures::PRELUDE).unwrap();
        if !row.setup.is_empty() {
            frontend.eval_source(row.setup).unwrap();
        }
        assert!(
            frontend.runtime_mut().take_output().is_empty(),
            "{} setup output",
            row.id
        );
        let result = frontend.eval_source(row.source);
        let visible = frontend.runtime().is_visible();
        match (&row.expected, result) {
            (Some(expected), Ok(mut values)) => {
                counts.0 += 1;
                let value = values.pop().expect("fixture source must yield a value");
                assert_eq!(observe::plain(&value), *expected, "{} typed value", row.id);
                assert_eq!(Some(visible), row.visible, "{} visibility", row.id);
                if !row.after.is_empty() {
                    frontend.eval_source(row.after).unwrap();
                }
                observe::effects(&mut frontend, row);
                observe::metadata(&mut frontend, &value, row);
            }
            (None, Err(error)) => {
                counts.1 += 1;
                assert_eq!(
                    Some(payload(&error, row.id).as_str()),
                    row.error,
                    "{} error",
                    row.id
                );
                observe::effects(&mut frontend, row);
            }
            (_, result) => panic!("{} outcome mismatch: {result:?}", row.id),
        }
        println!("EVAL_ROW\t{}\tPASS", row.id);
    }
    assert_eq!(counts, (52, 22));
    assert_eq!(ids.len(), 74);
}

fn payload(error: &EvalError, id: &str) -> String {
    match (id, error) {
        (
            "missing_expression" | "missing_evalq" | "computed_missing" | "bound_missing"
            | "omitted_forward",
            EvalError::MissingArgument(_),
        ) => error.to_string(),
        ("expr_force_first" | "expression_error", EvalError::Stop { message, .. }) => {
            message.clone()
        }
        ("unused_arg" | "ambiguous_arg", EvalError::Arity(message)) => message.clone(),
        ("break_outside" | "break_other_environment", EvalError::Control(message)) => {
            message.clone()
        }
        (
            "primitive_rejected"
            | "environment_invalid_enclos"
            | "invalid_both"
            | "argument_order"
            | "evalq_force_order"
            | "logical_envir"
            | "character_envir"
            | "numeric_length"
            | "numeric_na"
            | "numeric_range"
            | "partial_escalated",
            EvalError::Domain(message),
        ) => message.clone(),
        _ => panic!("{id} unexpected error variant {error:?}"),
    }
}

#[test]
fn frozen_warning_rows_have_exact_condition_messages_calls_and_classes() {
    let mut checked = 0;
    for row in fixtures::rows()
        .iter()
        .filter(|row| !row.warnings.is_empty())
    {
        let mut frontend = Frontend::default();
        frontend.eval_source(fixtures::PRELUDE).unwrap();
        if !row.setup.is_empty() {
            frontend.eval_source(row.setup).unwrap();
        }
        frontend.eval_source(".d3_warnings<-list()").unwrap();
        let source = format!("withCallingHandlers({},warning=function(w){{.d3_warnings[[length(.d3_warnings)+1L]]<<-w;invokeRestart('muffleWarning')}})", row.source);
        frontend.eval_source(&source).unwrap();
        assert_eq!(
            observe::last(&mut frontend, "length(.d3_warnings)"),
            Sexp::Integer(vec![Some(i32::try_from(row.warnings.len()).unwrap())]),
            "{} count",
            row.id
        );
        for (index, message) in row.warnings.iter().enumerate() {
            let prefix = format!(".d3_warnings[[{}]]", index + 1);
            assert_eq!(
                observe::last(&mut frontend, &format!("conditionMessage({prefix})")),
                observe::strings(&[message]),
                "{} message",
                row.id
            );
            assert_eq!(
                observe::last(&mut frontend, &format!("class({prefix})")),
                observe::strings(row.warning_classes[index]),
                "{} class",
                row.id
            );
            let call = observe::last(&mut frontend, &format!("conditionCall({prefix})"));
            assert_eq!(
                r_ust_eval::format_sexp(&call).unwrap(),
                row.warning_calls[index],
                "{} call",
                row.id
            );
        }
        assert!(
            frontend.runtime_mut().take_output().is_empty(),
            "{} muffled output",
            row.id
        );
        println!("EVAL_WARNING\t{}\tPASS", row.id);
        checked += 1;
    }
    assert_eq!(checked, 2);
}
