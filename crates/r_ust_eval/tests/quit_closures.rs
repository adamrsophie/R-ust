use r_ust_eval::{EvalError, Frontend, OutputEvent, QuitRequest, SaveAction};
use r_ust_sexp::Sexp;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

fn check(source: &str, status: i32, output: &str) -> Result {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source(source),
        Err(EvalError::Quit(QuitRequest {
            save: SaveAction::No,
            status,
            run_last: false,
        })),
        "{source}"
    );
    let actual: String = frontend
        .runtime_mut()
        .take_output()
        .into_iter()
        .map(|event| match event {
            OutputEvent::Stdout(text) => text,
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(actual, output, "{source}");
    assert_eq!(
        frontend.eval_source("1L")?,
        vec![Sexp::Integer(vec![Some(1)])]
    );
    Ok(())
}

#[test]
fn quit_closures_alias_and_higher_order_calls_unwind() -> Result {
    for source in [
        "f<-quit;f('no',7,FALSE);stop('BAD')",
        "f<-q;f('no',7,FALSE);stop('BAD')",
        "lapply('no',quit,status=7,runLast=FALSE);stop('BAD')",
        "do.call(quit,list('no',7,FALSE));stop('BAD')",
        "(function(f)f('no',7,FALSE))(q);stop('BAD')",
    ] {
        check(source, 7, "")?;
    }
    Ok(())
}

#[test]
fn quit_closures_formal_matching_and_lazy_forwarding() -> Result {
    for source in [
        "f<-function(...)q(...);f('no',st=6,run=FALSE)",
        "quit(save='no',s=6,run=FALSE)",
        "quit(status=6,'no',FALSE)",
        "(function(x)x)(quit('no',6,FALSE));stop('BAD')",
    ] {
        check(source, 6, "")?;
    }
    check("quit('no',,FALSE)", 0, "")?;
    check(
        "f<-function(x)cat('OK|');f(quit('no',3,FALSE));q('no',0,FALSE)",
        0,
        "OK|",
    )
}

#[test]
fn quit_closures_force_formals_once_in_wrapper_body_order() -> Result {
    check(
        "q(runLast={cat('R|');FALSE},status={cat('T|');9},save={cat('S|');'no'})",
        9,
        "S|T|R|",
    )
}

#[test]
fn quit_closures_are_not_caught_as_errors() -> Result {
    for source in [
        "tryCatch(quit('no',4,FALSE),error=function(e)stop('BAD'));stop('BAD')",
        "withCallingHandlers(q('no',4,FALSE),error=function(e)stop('BAD'));stop('BAD')",
        "f<-function(){on.exit(cat('BAD'));q('no',4,FALSE)};f()",
    ] {
        check(source, 4, "")?;
    }
    Ok(())
}

#[test]
fn quit_closures_keep_q_alias_when_one_binding_is_shadowed() -> Result {
    check(
        "quit<-function(...)cat('LOCAL|');quit();q('no',0,FALSE)",
        0,
        "LOCAL|",
    )?;
    check(
        "q<-function(...)cat('LOCAL|');q();quit('no',0,FALSE)",
        0,
        "LOCAL|",
    )
}

#[test]
fn quit_closures_introspection_exposes_real_wrapper() -> Result {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend.eval_source("typeof(quit);typeof(q);identical(q,quit)")?,
        vec![
            Sexp::String(vec![Some("closure".into())]),
            Sexp::String(vec![Some("closure".into())]),
            Sexp::Logical(vec![Some(true)]),
        ]
    );
    assert_eq!(frontend.eval_source("identical(formals(quit),formals(function(save='default',status=0,runLast=TRUE)NULL));identical(body(quit),quote(.Internal(quit(save,status,runLast))))")?, vec![Sexp::Logical(vec![Some(true)]);2]);
    Ok(())
}

#[test]
fn quit_closures_internal_uses_positional_values_and_exact_arity() -> Result {
    check(".Internal(quit(runLast='no',save=12,status=FALSE))", 12, "")?;
    for (source, expected) in [
        (".Internal(quit({cat('S|');'no'},{cat('T|');0}))", "S|T|"),
        (".Internal(quit('no',0,FALSE,{cat('X|');1}))", "X|"),
    ] {
        let mut frontend = Frontend::default();
        let error = frontend
            .eval_source(source)
            .expect_err("wrong internal arity must fail");
        assert!(
            error
                .to_string()
                .contains("passed to .Internal(quit) which requires 3"),
            "{error}"
        );
        let actual: String = frontend
            .runtime_mut()
            .take_output()
            .into_iter()
            .map(|event| match event {
                OutputEvent::Stdout(text) => text,
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(actual, expected);
    }
    Ok(())
}

#[test]
fn quit_closures_matching_rejects_before_forcing() {
    for source in [
        "q(bogus={cat('BAD');1})",
        "q(s={cat('BAD');'no'})",
        "q(save='no',save={cat('BAD');'no'})",
        "q('no',0,FALSE,{cat('BAD');1})",
    ] {
        let mut frontend = Frontend::default();
        let error = frontend
            .eval_source(source)
            .expect_err("matching must fail");
        assert!(!matches!(error, EvalError::Quit(_)), "{source}");
        assert!(frontend.runtime_mut().take_output().is_empty(), "{source}");
    }
}

#[test]
fn quit_closures_validate_after_all_three_formals_are_forced() {
    let mut frontend = Frontend::default();
    assert!(frontend
        .eval_source("q(save={cat('S|');'bad'},status={cat('T|');0},runLast={cat('R|');FALSE})")
        .is_err());
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![
            OutputEvent::Stdout("S|".into()),
            OutputEvent::Stdout("T|".into()),
            OutputEvent::Stdout("R|".into()),
        ]
    );
}
