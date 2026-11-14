use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn last(frontend: &mut Frontend, source: &str) -> Sexp {
    frontend.eval_source(source).unwrap().pop().unwrap()
}

#[test]
fn source_branches_and_order() {
    let cases = [
        ("n<-0L;withCallingHandlers({signalCondition(simpleError('A'));signalCondition(simpleError('B'));n},error=function(e)n<<-n+1L)", Sexp::Integer(vec![Some(2)])),
        ("f<-function(x=absent)function()x;h<-f();tryCatch(h(),error=function(e)'CAUGHT')", Sexp::String(vec![Some("CAUGHT".into())])),
        ("tryCatch(7L)", Sexp::Integer(vec![Some(7)])),
        ("tryCatch(stop('A'),error=function(e)e$message)", Sexp::String(vec![Some("A".into())])),
        ("tryCatch(stop('A'),simpleError=function(e)'S',error=function(e)'E')", Sexp::String(vec![Some("S".into())])),
        ("tryCatch(stop(simpleError('C')),error=function(e)e$message)", Sexp::String(vec![Some("C".into())])),
        ("f<-function(){on.exit(tryCatch(stop('SECOND'),error=function(e)NULL));stop('ORIGINAL')};tryCatch(f(),error=function(e)e$message)", Sexp::String(vec![Some("ORIGINAL".into())])),
    ];
    for (source, expected) in cases {
        let mut frontend = Frontend::default();
        assert_eq!(last(&mut frontend, source), expected, "{source}");
        assert_eq!(
            last(&mut frontend, "sys.nframe()"),
            Sexp::Integer(vec![Some(0)])
        );
    }
    let mut frontend = Frontend::default();
    assert_eq!(last(&mut frontend,
        "f<-function(){on.exit(cat('CLEAN'));withCallingHandlers(stop('A'),error=function(e)cat('CALL'))};tryCatch(f(),error=function(e){cat('EXIT');e$message})"),
        Sexp::String(vec![Some("A".into())]));
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("CALL".into()),
            OutputEvent::Stdout("CLEAN".into()),
            OutputEvent::Stdout("EXIT".into()),
        ]
    );
}

#[test]
fn cleanup_order_clear_and_replacement() {
    for (source, expected, output) in [
        ("f<-function(){on.exit(cat('A'));on.exit(cat('B'),add=TRUE);on.exit(cat('C'),add=TRUE,after=FALSE);7L};f()", 7, "CAB"),
        ("f<-function(){on.exit(cat('A'));on.exit();7L};f()", 7, ""),
        ("f<-function(){on.exit(return(8L));return(7L)};f()", 8, ""),
        ("f<-function(){on.exit(return(8L));on.exit(cat('B'),add=TRUE);return(7L)};f()", 8, "B"),
        ("f<-function(){on.exit(on.exit(cat('NEW')));on.exit(cat('OLD'),add=TRUE);7L};f()", 7, "OLD"),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(last(&mut frontend, source), Sexp::Integer(vec![Some(expected)]), "{source}");
        let output_actual = frontend.runtime_mut().take_output().into_iter().map(|event| match event {
            OutputEvent::Stdout(text) => text,
            event => panic!("unexpected event {event:?}"),
        }).collect::<String>();
        assert_eq!(output_actual, output, "{source}");
    }
}

#[test]
fn cleanup_failure_replaces_pending_error_without_duplicate_actions() {
    let mut frontend = Frontend::default();
    assert_eq!(last(&mut frontend,
        "f<-function(){on.exit({cat('A');stop('NEW')});on.exit(cat('B'),add=TRUE);stop('OLD')};tryCatch(f(),error=function(e)e$message)"),
        Sexp::String(vec![Some("NEW".into())]));
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("A".into()),
            OutputEvent::Stdout("B".into())
        ]
    );
}

#[test]
fn control_and_host_errors_are_not_conditions() {
    let mut frontend = Frontend::default();
    assert_eq!(
        last(
            &mut frontend,
            "f<-function()tryCatch(return(4L),error=function(e)99L);f()"
        ),
        Sexp::Integer(vec![Some(4)])
    );
    assert!(matches!(
        frontend.eval_source("tryCatch(q('no',7L,FALSE),error=function(e)99L)"),
        Err(EvalError::Quit(_))
    ));
    frontend
        .runtime_mut()
        .set_output_sink(|_| Err(std::io::Error::other("sink failed")));
    assert!(
        matches!(frontend.eval_source("tryCatch(cat('x'),error=function(e)99L)"), Err(EvalError::Output(message)) if message == "sink failed")
    );
}

#[test]
fn source_condition_identity_and_call_are_preserved() {
    let mut frontend = Frontend::default();
    assert_eq!(
        last(
            &mut frontend,
            "x<-simpleError('C',quote(f(1L)));tryCatch(stop(x),error=function(e)identical(e,x))"
        ),
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(last(&mut frontend,
        "f<-function()stop('C');tryCatch(f(),error=function(e)identical(conditionCall(e),quote(f())))"), Sexp::Logical(vec![Some(true)]));
    assert_eq!(last(&mut frontend,
        "f<-function()stop('C',call.=FALSE);tryCatch(f(),error=function(e)is.null(conditionCall(e)))"), Sexp::Logical(vec![Some(true)]));
}

#[test]
fn invalid_handlers_and_on_exit_flags_remain_language_errors() {
    for (source, message) in [
        (
            "tryCatch(1,function(e)e)",
            "condition handlers must be specified with a condition class",
        ),
        (
            ".Internal(.addCondHands(1,list(function(e)e),globalenv(),NULL,FALSE))",
            "bad handler data",
        ),
        (
            "f<-function()on.exit(1,add=NA);f()",
            "invalid 'add' argument",
        ),
        (
            "f<-function()on.exit(1,after=NA);f()",
            "invalid 'lifo' argument",
        ),
    ] {
        let mut frontend = Frontend::default();
        let wrapped = format!("tryCatch({{{source}}},error=function(e)conditionMessage(e))");
        assert_eq!(
            last(&mut frontend, &wrapped),
            Sexp::String(vec![Some(message.into())])
        );
    }
}

#[test]
fn nested_source_helpers_are_real_activations() {
    for (source, depth) in [
        ("tryCatch(sys.nframe())", 0),
        ("tryCatch(sys.nframe(),error=function(e)0L)", 0),
        ("withCallingHandlers(sys.nframe(),error=function(e)0L)", 0),
        ("tryCatch({f<-function()sys.nframe();f()})", 3),
        (
            "tryCatch({f<-function()sys.nframe();f()},error=function(e)0L)",
            5,
        ),
        (
            "withCallingHandlers({f<-function()sys.nframe();f()},error=function(e)0L)",
            2,
        ),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            last(&mut frontend, source),
            Sexp::Integer(vec![Some(depth)]),
            "{source}"
        );
    }
}

#[test]
fn default_stop_does_not_resignal_explicit_condition() {
    let mut frontend = Frontend::default();
    assert!(matches!(frontend.eval_source(
        "count<-0L;withCallingHandlers(stop(simpleError('C')),error=function(e)count<<-count+1L)"
    ), Err(EvalError::Stop { message, .. }) if message == "C"));
    assert_eq!(last(&mut frontend, "count"), Sexp::Integer(vec![Some(1)]));
}

#[test]
fn registered_nargs_counts_supplied_arguments_in_its_lexical_frame() {
    for (source, expected) in [
        ("nargs()", None),
        ("f<-function(x=1L,...)nargs();f()", Some(0)),
        ("f<-function(x=1L,...)nargs();f(,2L)", Some(2)),
        ("n<-nargs;f<-function(...)n();f(1L,2L)", Some(2)),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            last(&mut frontend, source),
            Sexp::Integer(vec![expected]),
            "{source}"
        );
    }
}

#[test]
fn registered_nargs_evaluates_actuals_then_reports_the_alias_call() {
    let mut frontend = Frontend::default();
    let value = last(&mut frontend,
        "n<-nargs;tryCatch(n({cat('ARG');1L}),error=function(e)identical(e$call,quote(n({cat('ARG');1L}))))");
    assert_eq!(value, Sexp::Logical(vec![Some(true)]));
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("ARG".into())]
    );
}

#[test]
fn source_finally_runs_after_return_handler_and_cleanup_scopes() {
    for (source, expected, effects) in [
        ("f<-function(){oe<-on.exit;oe(cat('CLEAN'));cat('BODY');9L};tryCatch(f(),finally=cat('FINALLY'))", Sexp::Integer(vec![Some(9)]), vec!["BODY", "CLEAN", "FINALLY"]),
        ("tryCatch(stop('A'),error=function(e){cat('HANDLER');e$message},finally=cat('FINALLY'))", Sexp::String(vec![Some("A".into())]), vec!["HANDLER", "FINALLY"]),
        ("tryCatch(tryCatch(stop('A'),error=function(e)stop('B'),finally=cat('INNER')),error=function(e)e$message,finally=cat('OUTER'))", Sexp::String(vec![Some("B".into())]), vec!["INNER", "OUTER"]),
    ] {
        let mut frontend = Frontend::default();
        let value = last(&mut frontend, source);
        assert_eq!(value, expected, "{source}");
        assert_eq!(frontend.runtime_mut().take_output(), effects.into_iter().map(|text| OutputEvent::Stdout(text.into())).collect::<Vec<_>>(), "{source}");
    }
}

#[test]
fn condition_wrappers_preserve_finally_control_and_computed_heads() {
    for (source, expected, effects) in [
        ("f<-function(){on.exit(cat('CLEAN'));tryCatch({cat('BODY');1L},finally={cat('FINALLY');return(2L)});99L};f()", Sexp::Integer(vec![Some(2)]), "BODYFINALLYCLEAN"),
        ("tryCatch(tryCatch({cat('BODY');1L},finally={cat('FINALLY');stop('FINAL')}),error=function(e)conditionMessage(e))", Sexp::String(vec![Some("FINAL".into())]), "BODYFINALLY"),
        ("f<-function(){on.exit(cat('CLEAN'));tryCatch(return(1L),finally={cat('FINALLY');stop('FINAL')})};tryCatch(f(),error=function(e)conditionMessage(e))", Sexp::String(vec![Some("FINAL".into())]), "FINALLYCLEAN"),
        ("n<-0L;for(i in 1:3){tryCatch({n<-n+1L;break},error=function(e)99L,finally=cat('F'));n<-99L};n", Sexp::Integer(vec![Some(1)]), "F"),
        ("n<-0L;for(i in 1:3){tryCatch({n<-n+1L;next},error=function(e)99L,finally=cat('F'));n<-99L};n", Sexp::Integer(vec![Some(3)]), "FFF"),
        ("n<-0L;for(i in 1:3){withCallingHandlers({n<-n+1L;break},error=function(e)99L);n<-99L};n", Sexp::Integer(vec![Some(1)]), ""),
        ("n<-0L;for(i in 1:3){withCallingHandlers({n<-n+1L;next},error=function(e)99L);n<-99L};n", Sexp::Integer(vec![Some(3)]), ""),
        ("tc<-tryCatch;tc(stop('A'),error=function(e){cat('H');conditionMessage(e)})", Sexp::String(vec![Some("A".into())]), "H"),
        ("(function()tryCatch)()(stop('B'),error=function(e){cat('H');conditionMessage(e)})", Sexp::String(vec![Some("B".into())]), "H"),
        ("wc<-withCallingHandlers;n<-0L;wc({signalCondition(simpleError('C'));n},error=function(e){cat('H');n<<-n+1L})", Sexp::Integer(vec![Some(1)]), "H"),
        ("n<-0L;(function()withCallingHandlers)()({signalCondition(simpleError('C'));n},error=function(e){cat('H');n<<-n+1L})", Sexp::Integer(vec![Some(1)]), "H"),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(last(&mut frontend, source), expected, "{source}");
        assert_eq!(last(&mut frontend, "sys.nframe()"), Sexp::Integer(vec![Some(0)]));
        let actual = frontend.runtime_mut().take_output().into_iter().map(|event| match event {
            OutputEvent::Stdout(text) => text,
            event => panic!("unexpected event {event:?}"),
        }).collect::<String>();
        assert_eq!(actual, effects, "{source}");
    }
}
