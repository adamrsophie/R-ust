use crate::language_print::{format_deparse, Profile};
use crate::{CallArgument, EvalError, Frontend, RuntimeState, Sexp};
use r_ust_sexp::{EvaluationOrigin, PromiseState};

fn profile(control: u32, lines: Option<usize>) -> Profile {
    Profile {
        cutoff: 20,
        backtick: true,
        control,
        lines,
        utf8: false,
    }
}

fn call(name: &str, values: Vec<Sexp>) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Symbol(name.into())),
        arguments: values
            .into_iter()
            .map(|value| CallArgument { name: None, value })
            .collect(),
    }
}

#[test]
fn deparse_delayed_promise_retains_expression_then_normal_deparse_forces_once(
) -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    runtime
        .current
        .insert("payload".into(), &Sexp::Integer(vec![Some(41)]))?;
    let expression = Sexp::Symbol("payload".into());
    let state = PromiseState::new(
        &expression,
        &runtime.current,
        EvaluationOrigin::Global,
        true,
    )?;
    let value = Sexp::Promise {
        state: state.clone(),
    };
    let (lines, incomplete) = format_deparse(&mut runtime, &value, profile(1089 | 32, None))?;
    assert_eq!(lines, ["<promise: payload>"]);
    assert!(incomplete);
    assert!(state.environment()?.is_some());
    assert_eq!(state.expression()?, expression);
    assert_eq!(
        format_deparse(&mut runtime, &value, profile(1089, None))?,
        (vec!["41L".into()], false)
    );
    assert!(state.environment()?.is_none());
    runtime
        .current
        .insert("payload".into(), &Sexp::Integer(vec![Some(42)]))?;
    assert_eq!(
        format_deparse(&mut runtime, &value, profile(1089, None))?.0,
        ["41L"]
    );
    assert!(runtime.take_output().is_empty());
    Ok(())
}

#[test]
fn deparse_line_limit_does_not_force_a_later_promise() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let state = PromiseState::new(
        &Sexp::Symbol("UNFORCED".into()),
        &runtime.current,
        EvaluationOrigin::Global,
        true,
    )?;
    let value = call(
        "{",
        vec![Sexp::Promise {
            state: state.clone(),
        }],
    );
    assert_eq!(
        format_deparse(&mut runtime, &value, profile(1089, Some(1)))?,
        (vec!["{".into()], false)
    );
    assert!(state.environment()?.is_some());
    assert!(format_deparse(&mut runtime, &value, profile(1089, None)).is_err());
    assert!(runtime.control.contexts.is_empty());
    Ok(())
}

#[test]
fn deparse_line_limit_does_not_inspect_a_later_invalid_tag() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let value = Sexp::Call {
        function: Box::new(Sexp::Symbol("f".into())),
        arguments: vec![
            CallArgument {
                name: None,
                value: Sexp::Symbol("abcdefghijklmnopqrstuvwxyz".into()),
            },
            CallArgument {
                name: Some("invalid\0tag".into()),
                value: Sexp::Nil,
            },
        ],
    };
    assert_eq!(
        format_deparse(&mut runtime, &value, profile(1089, Some(1)))?.0,
        ["f(abcdefghijklmnopqrstuvwxyz, "]
    );
    assert!(matches!(
        format_deparse(&mut runtime, &value, profile(1089, None)),
        Err(EvalError::Type(_))
    ));
    Ok(())
}

#[test]
fn format_source_bindings_preserve_closure_formals() {
    let mut frontend = Frontend::default();
    for (name, expected) in [
        ("sprintf", vec!["fmt", "..."]),
        (
            "deparse",
            vec!["expr", "width.cutoff", "backtick", "control", "nlines"],
        ),
        (".deparseOpts", vec!["control"]),
        ("sQuote", vec!["x", "q"]),
        ("ngettext", vec!["n", "msg1", "msg2", "domain"]),
        ("l10n_info", vec![]),
    ] {
        let values = frontend.eval_source(name).unwrap();
        let Sexp::Closure {
            parameters,
            attributes,
            ..
        } = &values[0]
        else {
            panic!("{name} closure")
        };
        assert_eq!(
            parameters
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            expected,
            "{name}"
        );
        assert!(attributes.is_empty(), "{name} unexpected attributes");
    }
}
