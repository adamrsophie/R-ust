use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::Sexp;

fn evaluate(source: &str) -> (Sexp, Vec<OutputEvent>) {
    let mut frontend = Frontend::default();
    let value = frontend.eval_source(source).unwrap().pop().unwrap();
    (value, frontend.runtime_mut().take_output())
}

#[test]
fn native_fault_calls_keep_their_source_selected_owner() {
    for source in [
        "f<-function()absent;tryCatch(f(),error=function(e)identical(e$call,quote(f())))",
        "f<-function(x)x;g<-function()f(absent);tryCatch(g(),error=function(e)identical(e$call,quote(g())))",
        "tryCatch(absent(),error=function(e)identical(e$call,quote(absent())))",
        "f<-function(g)g();tryCatch(f(),error=function(e)identical(e$call,quote(g())))",
        "f<-function(x)1L;tryCatch(f(1L,2L),error=function(e)identical(e$call,quote(f(1L,2L))))",
        "tryCatch(.Primitive('sys.frame'),error=function(e)identical(e$call,quote(.Primitive('sys.frame'))))",
        "p<-.Primitive;tryCatch(p('sys.frame'),error=function(e)identical(e$call,quote(p('sys.frame'))))",
        "tryCatch(.Primitive('.Primitive')('sys.frame'),error=function(e)identical(e$call,quote(.Primitive('.Primitive')('sys.frame'))))",
        "tryCatch(.Internal(1L),error=function(e)identical(e$call,quote(.Internal(1L))))",
        "tryCatch(.Internal(new.env(,globalenv(),29L)),error=function(e)identical(e$call,quote(.Internal(new.env(,globalenv(),29L)))))",
        "options(warn=2L);tryCatch(.Internal(body(1L)),error=function(e)identical(e$call,quote(body(1L))))",
        "options(warn=2L);tryCatch(.Internal(formals(1L)),error=function(e)identical(e$call,quote(formals(1L))))",
        "f<-function()sys.frame(999L);tryCatch(f(),error=function(e)identical(e$call,quote(sys.frame(999L))))",
        "f<-function()stop('A',call.=FALSE);tryCatch(f(),error=function(e)is.null(e$call))",
    ] {
        assert_eq!(evaluate(source).0, Sexp::Logical(vec![Some(true)]), "{source}");
    }
}

#[test]
fn missing_argument_conditions_keep_native_subclasses() {
    for (source, subclass) in [
        (
            "f<-function(x)x;tryCatch(f(),error=function(e)class(e))",
            "evalError",
        ),
        (
            "f<-function(expr){on.exit(expr);3L};tryCatch(f(),error=function(e)class(e))",
            "evalError",
        ),
        (
            "f<-function(g)g();tryCatch(f(),error=function(e)class(e))",
            "getMissingError",
        ),
    ] {
        let expected = [subclass, "missingArgError", "error", "condition"]
            .map(|name| Some(name.into()))
            .to_vec();
        assert_eq!(evaluate(source).0, Sexp::String(expected), "{source}");
    }
}

#[test]
fn raw_on_exit_dots_fail_at_cleanup_without_forcing_supplied_dots() {
    for invocation in ["f(cat('FORCED'))", "f(,add=FALSE)"] {
        let source = format!(
            "f<-function(...){{on.exit(...);cat('REGISTERED');1L}};tryCatch({invocation},error=function(e)e$message)"
        );
        assert_eq!(
            evaluate(&source),
            (
                Sexp::String(vec![Some("'...' used in an incorrect context".into())]),
                vec![OutputEvent::Stdout("REGISTERED".into())],
            )
        );
    }
}

#[test]
fn raw_on_exit_flags_run_in_formal_order_before_body_and_cleanup() {
    let (value, effects) = evaluate(
        "f<-function(){on.exit(after={cat('AFTER');FALSE},expr=cat('CLEAN'),add={cat('ADD');TRUE});cat('BODY');5L};f()"
    );
    assert_eq!(value, Sexp::Integer(vec![Some(5)]));
    assert_eq!(
        effects,
        ["ADD", "AFTER", "BODY", "CLEAN"].map(|text| OutputEvent::Stdout(text.into()))
    );
}

#[test]
fn raw_on_exit_empty_expr_and_invalid_add_preserve_evaluation_order() {
    for (source, expected, output) in [
        (
            "f<-function(){on.exit(,add=TRUE);cat('REGISTERED');4L};f()",
            Sexp::Integer(vec![Some(4)]),
            "REGISTERED",
        ),
        (
            "f<-function(){on.exit(after={cat('AFTER');TRUE},expr=cat('CLEAN'),add={cat('ADD');NA});cat('BODY')};tryCatch(f(),error=function(e)e$message)",
            Sexp::String(vec![Some("invalid 'add' argument".into())]),
            "ADD",
        ),
    ] {
        assert_eq!(evaluate(source), (expected, vec![OutputEvent::Stdout(output.into())]));
    }
}

#[test]
fn a_propagated_simple_fault_signals_calling_handler_once() {
    let source = "n<-0L;f<-function()absent;tryCatch(withCallingHandlers(f(),error=function(e)n<<-n+1L),error=function(e)NULL);n";
    assert_eq!(evaluate(source).0, Sexp::Integer(vec![Some(1)]));
    let mut frontend = Frontend::default();
    assert_eq!(frontend.eval_source("n<-0L;f<-function(){on.exit(cat('CLEAN'));absent};withCallingHandlers(f(),error=function(e){n<<-n+1L;cat(n)})"), Err(r_ust_eval::EvalError::UndefinedSymbol("absent".into())));
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![
            OutputEvent::Stdout("1".into()),
            OutputEvent::Stdout("CLEAN".into())
        ]
    );
    assert_eq!(
        frontend.eval_source("n").unwrap(),
        vec![Sexp::Integer(vec![Some(1)])]
    );
}

#[test]
fn malformed_handler_leaf_preserves_native_call_class_and_message() {
    for (source, call) in [
        (
            ".Internal(.addCondHands(1,list(function(e)e),globalenv(),NULL,FALSE))",
            "doTryCatch(return(expr),name,parentenv,handler)",
        ),
        ("f()", "f()"),
    ] {
        let setup =
            "f<-function().Internal(.addCondHands(1,list(function(e)e),globalenv(),NULL,FALSE));";
        let source_call = format!(
            "{setup}tryCatch({source},error=function(e)identical(conditionCall(e),quote({call})))"
        );
        assert_eq!(
            evaluate(&source_call).0,
            Sexp::Logical(vec![Some(true)]),
            "{source_call}"
        );
        let source_class = format!("{setup}tryCatch({source},error=function(e)class(e))");
        assert_eq!(
            evaluate(&source_class).0,
            Sexp::String(
                ["simpleError", "error", "condition"]
                    .map(|value| Some(value.into()))
                    .to_vec()
            ),
            "{source_class}"
        );
        let source_message =
            format!("{setup}tryCatch({source},error=function(e)conditionMessage(e))");
        assert_eq!(
            evaluate(&source_message).0,
            Sexp::String(vec![Some("bad handler data".into())]),
            "{source_message}"
        );
    }
}
