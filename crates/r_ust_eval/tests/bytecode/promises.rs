use super::*;
use r_ust_sexp::{EvaluationOrigin, PromiseState};

#[test]
fn compiled_symbol_missing_and_interpreted_promise_faults_keep_distinct_classes() -> Result {
    let mut frontend = Frontend::default();
    let function = compile(&mut frontend, "function(x)x")?;
    bind(&frontend, "compiled", &function)?;
    assert_eq!(
        frontend
            .eval_source("tryCatch(compiled(),error=function(e)class(e))")?
            .pop(),
        Some(Sexp::String(
            ["getvarError", "missingArgError", "error", "condition"]
                .map(|s| Some(s.into()))
                .to_vec()
        ))
    );
    assert_eq!(
        frontend
            .eval_source(
                "forward<-function(y)compiled(y);tryCatch(forward(),error=function(e)class(e))"
            )?
            .pop(),
        Some(Sexp::String(
            ["evalError", "missingArgError", "error", "condition"]
                .map(|s| Some(s.into()))
                .to_vec()
        ))
    );
    assert_eq!(
        frontend
            .eval_source("tryCatch(eval(),error=function(e)class(e))")?
            .pop(),
        Some(Sexp::String(
            ["getvarError", "missingArgError", "error", "condition"]
                .map(|s| Some(s.into()))
                .to_vec()
        ))
    );
    assert_eq!(
        frontend.eval_source("sys.nframe()")?.pop(),
        Some(Sexp::Integer(vec![Some(0)]))
    );
    Ok(())
}

#[test]
fn compiled_promises_force_once_and_source_inspection_never_forces() -> Result {
    let mut frontend = Frontend::default();
    frontend.eval_source("ticks<-0L")?;
    let frame = owner(frontend.runtime())?;
    let expr = r_ust_parse::Parser
        .parse_all("{ticks<-ticks+1L;7L}")
        .map_err(|error| r_ust_eval::EvalError::Parse(error.message))?
        .expressions
        .remove(0);
    let inner = PromiseState::new(&expr, &frame, EvaluationOrigin::Global, true)?;
    frame.insert("x".into(), &Sexp::Promise { state: inner })?;
    let closure = compile(&mut frontend, "function()x")?;
    let code = body_code(&closure).clone();
    let outer = PromiseState::new(
        &Sexp::Bytecode(code.clone()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    let env = frame.child()?;
    env.insert(
        "p".into(),
        &Sexp::Promise {
            state: outer.clone(),
        },
    )?;
    bind(&frontend, "probe", &Sexp::Environment(env))?;
    assert_eq!(
        frontend.eval_source("substitute(p,probe)")?.pop(),
        Some(Sexp::Symbol("x".into()))
    );
    assert_eq!(
        frontend.runtime().get("ticks")?,
        Some(Sexp::Integer(vec![Some(0)]))
    );
    for _ in 0..2 {
        assert_eq!(
            frontend.runtime_mut().eval(&Sexp::Promise {
                state: outer.clone()
            })?,
            Sexp::Integer(vec![Some(7)])
        );
    }
    assert_eq!(
        frontend.runtime().get("ticks")?,
        Some(Sexp::Integer(vec![Some(1)]))
    );
    let Sexp::Bytecode(stored) = outer.expression()? else {
        panic!("actual compiled promise")
    };
    assert!(stored.same_object(&code));
    assert_eq!(
        frontend.eval_source("substitute(p,probe)")?.pop(),
        Some(Sexp::Symbol("x".into()))
    );
    Ok(())
}

#[test]
fn failed_compiled_promise_can_restart_after_binding_repair() -> Result {
    let mut frontend = Frontend::default();
    let closure = compile(&mut frontend, "function()later")?;
    let frame = owner(frontend.runtime())?;
    let promise = PromiseState::new(
        &Sexp::Bytecode(body_code(&closure).clone()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    assert_eq!(
        frontend.runtime_mut().eval(&Sexp::Promise {
            state: promise.clone()
        }),
        Err(r_ust_eval::EvalError::UndefinedSymbol("later".into()))
    );
    frame.insert("later".into(), &Sexp::Integer(vec![Some(9)]))?;
    assert_eq!(
        frontend
            .runtime_mut()
            .eval(&Sexp::Promise { state: promise })?,
        Sexp::Integer(vec![Some(9)])
    );
    assert_eq!(
        frontend.eval_source("sys.nframe()")?.pop(),
        Some(Sexp::Integer(vec![Some(0)]))
    );
    Ok(())
}
