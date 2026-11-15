use super::*;
use r_ust_eval::OutputEvent;
use r_ust_sexp::ClosureEnvironment;

#[test]
fn guard_forces_promised_callable_once_before_any_operand() -> Result {
    let mut frontend = Frontend::default();
    let function = compile(&mut frontend, "function(x,.Internal).Internal(body(x))")?;
    bind(&frontend, "f", &function)?;
    assert_eq!(
        frontend
            .eval_source(
                "f({cat('BAD|');function()1L},{cat('FORCE|');function(call){cat('CALL|');77L}})"
            )?
            .pop(),
        Some(Sexp::Integer(vec![Some(77)]))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![
            OutputEvent::Stdout("FORCE|".into()),
            OutputEvent::Stdout("CALL|".into())
        ]
    );
    assert_eq!(
        frontend
            .eval_source("f({cat('ARG|');function()1L},5L)")?
            .pop(),
        Some(Sexp::Integer(vec![Some(1)]))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stdout("ARG|".into())]
    );
    assert_eq!(
        frontend
            .eval_source("tryCatch(f(function()1L),error=function(e)conditionMessage(e))")?
            .pop(),
        Some(Sexp::String(vec![Some(
            "argument \".Internal\" is missing, with no default".into()
        )]))
    );
    Ok(())
}

#[test]
fn absent_guard_binding_errors_without_evaluating_operand() -> Result {
    let mut frontend = Frontend::default();
    let mut function = compile(&mut frontend, "function(x).Internal(body(x))")?;
    let Some(Sexp::Environment(empty)) = frontend.eval_source("new.env(parent=emptyenv())")?.pop()
    else {
        panic!("environment")
    };
    let Sexp::Closure { environment, .. } = &mut function else {
        panic!("closure")
    };
    *environment = ClosureEnvironment::Captured(empty);
    bind(&frontend, "g", &function)?;
    assert_eq!(
        frontend
            .eval_source(
                "tryCatch(g({cat('BAD|');function()1L}),error=function(e)conditionMessage(e))"
            )?
            .pop(),
        Some(Sexp::String(vec![Some(
            "could not find function \".Internal\"".into()
        )]))
    );
    assert!(frontend.runtime_mut().take_output().is_empty());
    assert_eq!(
        frontend.eval_source("sys.nframe()")?.pop(),
        Some(Sexp::Integer(vec![Some(0)]))
    );
    Ok(())
}
