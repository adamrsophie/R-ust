use super::*;
use r_ust_eval::{EvalError, OutputEvent, SaveAction};

#[test]
fn compiled_internal_eval_restores_return_error_and_warning_transfers() -> Result {
    let mut frontend = Frontend::default();
    let evaluator = compile(&mut frontend, "function(x,e,n).Internal(eval(x,e,n))")?;
    bind(&frontend, "run", &evaluator)?;
    for (source, expected) in [
        ("(function(){run(quote(return(15L)),environment(),baseenv());99L})()", 99),
        ("tryCatch(run(quote(stop('C')),globalenv(),baseenv()),error=function(e)21L)", 21),
        ("withCallingHandlers(run(quote({warning('W');23L}),globalenv(),baseenv()),warning=function(w)invokeRestart('muffleWarning'))", 23),
    ] {
        assert_eq!(frontend.eval_source(source)?.pop(), Some(Sexp::Integer(vec![Some(expected)])));
        assert_eq!(frontend.eval_source("sys.nframe()")?.pop(), Some(Sexp::Integer(vec![Some(0)])));
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
    Ok(())
}

#[test]
fn quit_during_compiled_promise_forcing_skips_later_cleanup_and_keeps_sdk_usable() -> Result {
    let mut frontend = Frontend::default();
    let force = frontend.runtime().get("force")?.ok_or("force")?;
    assert!(matches!(body_code(&force).source()?, Sexp::Symbol(_)));
    let result = frontend.eval_source("f<-function(){on.exit(force({cat('FIRST|');quit(save='no',status=6L,runLast=FALSE)}));on.exit(cat('BAD|'),add=TRUE);stop('body')};tryCatch(f(),error=function(e){cat('HANDLER|');99L})");
    let Err(EvalError::Quit(request)) = result else {
        panic!("Quit request required")
    };
    assert_eq!(request.save, SaveAction::No);
    assert_eq!(request.status, 6);
    assert!(!request.run_last);
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![OutputEvent::Stdout("FIRST|".into())]
    );
    assert_eq!(
        frontend.eval_source("sys.nframe()")?.pop(),
        Some(Sexp::Integer(vec![Some(0)]))
    );
    assert_eq!(
        frontend.eval_source("force(31L)")?.pop(),
        Some(Sexp::Integer(vec![Some(31)]))
    );
    Ok(())
}

#[test]
fn constant_pool_retains_list_after_returned_alias_mutation() -> Result {
    let mut frontend = Frontend::default();
    let value = frontend.eval_source("list(1L)")?.pop().ok_or("list")?;
    let bytecode = raw(
        frontend.runtime(),
        vec![12, 16, 1, 1],
        vec![Sexp::Nil, value],
    )?;
    bind(&frontend, "program", &bytecode)?;
    frontend.eval_source("alias<-eval(program);alias[[1L]]<-9L")?;
    assert_eq!(
        frontend.eval_source("eval(program)[[1L]]")?.pop(),
        Some(Sexp::Integer(vec![Some(1)]))
    );
    assert_eq!(
        frontend.eval_source("alias[[1L]]")?.pop(),
        Some(Sexp::Integer(vec![Some(9)]))
    );
    Ok(())
}
