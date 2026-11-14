use super::*;
use crate::Frontend;

#[test]
fn control_and_scope_stacks_restore_after_all_exits() {
    for (index, source) in [
        "(function(){f<-function(x)x;f(return(42L));99L})()",
        "f<-function(x)x;repeat{f(break)}",
        "f<-function(x)x;for(i in 1:2)f(next)",
        "f<-function(x)x;while(TRUE)f(break)",
        "f<-function(x){private<-1L;x};f(missing_name)",
        "with(list(private=1L),missing_name)",
        "(function(){while(TRUE)missing_name})()",
        "(function(){repeat{quit(status=0)}})()",
    ]
    .into_iter()
    .enumerate()
    {
        let mut frontend = Frontend::default();
        let origin = frontend.runtime().control.origin.clone();
        let result = frontend.eval_source(source);
        assert_eq!(result.is_ok(), index < 4, "{source}: {result:?}");
        assert!(
            matches!(
                &result,
                Ok(_) | Err(EvalError::UndefinedSymbol(_) | EvalError::Quit(_))
            ),
            "{source}: {result:?}"
        );
        let runtime = frontend.runtime();
        assert_eq!(runtime.current, runtime.global, "{source}");
        assert_eq!(runtime.control.origin, origin, "{source}");
        assert!(runtime.control.contexts.is_empty(), "{source}");
        assert!(runtime.get("private").unwrap().is_none(), "{source}");
        assert_eq!(
            frontend.eval_source("42L").unwrap(),
            [Sexp::Integer(vec![Some(42)])]
        );
    }
}

#[test]
fn data_masks_own_local_control_and_forward_promises() {
    for (source, expected) in [
        ("(function(){with(list(a=1L),return(3L));99L})()", 99),
        (
            "(function(){f<-function(x)with(list(a=1L),x);f(return(3L));99L})()",
            3,
        ),
        (
            "f<-function(x)with(list(a=1L),x);for(i in 1:3){f(break);cat('NO')};42L",
            42,
        ),
        (
            "(function(){subset(data.frame(a=1:3),return(TRUE));99L})()",
            99,
        ),
        (
            "(function(){f<-function(x)subset(data.frame(a=1:3),x);f(return(3L));99L})()",
            3,
        ),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(source).unwrap().last(),
            Some(&Sexp::Integer(vec![Some(expected)])),
            "{source}"
        );
        assert!(frontend.runtime_mut().take_output().is_empty(), "{source}");
        assert_eq!(frontend.runtime().current, frontend.runtime().global);
        assert!(frontend.runtime().control.contexts.is_empty());
    }
    for signal in ["break", "next"] {
        let mut frontend = Frontend::default();
        let origin = frontend.runtime().control.origin.clone();
        let source =
            format!("(function(){{for(i in 1:3){{with(list(a=1L),{signal});cat('NO')}};42L}})()");
        assert_eq!(
            frontend.eval_source(&source),
            Err(EvalError::Control(
                "no loop for break/next, jumping to top level".into()
            ))
        );
        assert!(frontend.runtime_mut().take_output().is_empty());
        assert_eq!(frontend.runtime().current, frontend.runtime().global);
        assert_eq!(frontend.runtime().control.origin, origin);
        assert!(frontend.runtime().control.contexts.is_empty());
    }
}
