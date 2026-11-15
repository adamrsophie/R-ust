use super::*;
use r_ust_eval::EvalError;

#[test]
fn instructions_execute_instead_of_source_and_versions_select_fallback() -> Result {
    let mut runtime = RuntimeState::default();
    runtime.set_bytecode_enabled(true);
    for version in [8, 9, 10, 11, 12, 13] {
        let value = raw(
            &runtime,
            vec![version, 16, 1, 1],
            vec![
                Sexp::Integer(vec![Some(100)]),
                Sexp::Integer(vec![Some(42)]),
            ],
        )?;
        let expected = if (9..=12).contains(&version) { 42 } else { 100 };
        assert_eq!(runtime.eval(&value)?, Sexp::Integer(vec![Some(expected)]));
        runtime.set_bytecode_enabled(false);
        assert_eq!(runtime.eval(&value)?, Sexp::Integer(vec![Some(100)]));
        runtime.set_bytecode_enabled(true);
    }
    for words in [vec![8, 0], vec![12, 17, 1]] {
        let value = raw(&runtime, words, vec![])?;
        assert_eq!(runtime.eval(&value)?, Sexp::Nil);
    }
    let sentinel = raw(&runtime, vec![12, 0], vec![Sexp::Integer(vec![Some(100)])])?;
    assert_eq!(runtime.eval(&sentinel)?, Sexp::Integer(vec![Some(100)]));
    let reached = raw(
        &runtime,
        vec![12, 0, 1],
        vec![Sexp::Integer(vec![Some(100)])],
    )?;
    assert_eq!(
        runtime.eval(&reached),
        Err(EvalError::Domain("byte code version mismatch".into()))
    );
    runtime.set_bytecode_enabled(false);
    assert_eq!(runtime.eval(&reached)?, Sexp::Integer(vec![Some(100)]));
    assert!(runtime.take_output().is_empty());
    Ok(())
}

#[test]
fn malformed_streams_fail_checked_decoder_and_do_not_run_source() -> Result {
    let mut runtime = RuntimeState::default();
    runtime.set_bytecode_enabled(true);
    for (words, message) in [
        (vec![12, 16], "truncated instruction"),
        (vec![12, 16, 2, 1], "constant index out of bounds"),
        (vec![12, 20, 0, 1], "symbol constant required"),
        (vec![12, 33, 1], "operand stack underflow"),
        (vec![12, 1], "unbalanced return"),
        (vec![12, 17], "fallthrough or invalid jump"),
        (vec![12, 999, 1], "unknown instruction code"),
    ] {
        let value = raw(&runtime, words, vec![Sexp::Integer(vec![Some(100)])])?;
        assert_eq!(
            runtime.eval(&value),
            Err(EvalError::Type(format!("invalid bytecode: {message}")))
        );
    }
    let unsupported = raw(
        &runtime,
        vec![12, 2, 1],
        vec![Sexp::Integer(vec![Some(100)])],
    )?;
    assert_eq!(
        runtime.eval(&unsupported),
        Err(EvalError::NotImplemented("bytecode opcode"))
    );
    assert_eq!(
        runtime.eval(&Sexp::Integer(vec![Some(9)]))?,
        Sexp::Integer(vec![Some(9)])
    );
    Ok(())
}

#[test]
fn nested_internal_frames_and_dynamic_visibility_survive_calls() -> Result {
    let mut frontend = Frontend::default();
    let nested = compile(
        &mut frontend,
        "function(x) .Internal(bitwiseNot(.Internal(bitwiseNot(x))))",
    )?;
    bind(&frontend, "nested", &nested)?;
    assert_eq!(
        frontend.eval_source("nested(3L)")?.pop(),
        Some(Sexp::Integer(vec![Some(3)]))
    );
    let evaluator = compile(&mut frontend, "function(x,e,n) .Internal(eval(x,e,n))")?;
    bind(&frontend, "run", &evaluator)?;
    assert_eq!(
        frontend
            .eval_source("run(quote(invisible(7L)),globalenv(),baseenv())")?
            .pop(),
        Some(Sexp::Integer(vec![Some(7)]))
    );
    assert!(!frontend.runtime().is_visible());
    assert_eq!(
        frontend
            .eval_source("run(quote(8L),globalenv(),baseenv())")?
            .pop(),
        Some(Sexp::Integer(vec![Some(8)]))
    );
    assert!(frontend.runtime().is_visible());
    assert!(frontend.runtime_mut().take_output().is_empty());
    Ok(())
}
