use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[path = "formals_explicit/printing.rs"]
mod printing;
#[path = "formals_explicit/sdk.rs"]
mod sdk;

#[test]
fn explicit_formals_print_required_defaults_and_dots_without_forcing() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source_printed(concat!(
            "formals(function(x,y=1L,z=x+1L,...)NULL); ",
            "formals(function(unrelated=stop('UNFORCED'))NULL, envir=stop('UNFORCED'))"
        ))
        .unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        vec![
            OutputEvent::Stdout("$x\n\n\n$y\n[1] 1\n\n$z\nx + 1L\n\n$...\n\n\n".into()),
            OutputEvent::Stdout("$unrelated\nstop(\"UNFORCED\")\n\n".into()),
        ]
    );
}

#[test]
fn formals_retains_its_real_defaults_and_unsupported_lookup_boundaries() {
    let mut frontend = Frontend::default();
    frontend.eval_source_printed("formals(formals)").unwrap();
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout(
            "$fun\nsys.function(sys.parent())\n\n$envir\nparent.frame()\n\n".into()
        )]
    );
    for (source, error) in [
        (
            "formals()",
            EvalError::UndefinedFunction("sys.function".into()),
        ),
        (
            "formals('identity')",
            EvalError::UndefinedFunction("get".into()),
        ),
        (
            "formals('identity',envir=stop('UNFORCED'))",
            EvalError::UndefinedFunction("get".into()),
        ),
        (
            "f<-function(x)formals(x);f()",
            EvalError::MissingArgument("x".into()),
        ),
    ] {
        assert_eq!(frontend.eval_source(source), Err(error), "{source}");
        assert_eq!(frontend.runtime_mut().take_output(), []);
    }
}

#[test]
fn formals_aliases_and_internal_kernel_keep_primitives_silent_and_warn_for_other_values() {
    let mut frontend = Frontend::default();
    frontend.eval_source("options(warn=1); f<-formals").unwrap();
    assert_eq!(
        frontend
            .eval_source("f(sum);(identity(f))(.Primitive('quote')); .Internal(formals(1L))")
            .unwrap(),
        [Sexp::Nil, Sexp::Nil, Sexp::Nil]
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stderr(
            "Warning: argument is not a function\n".into()
        )]
    );
    frontend.eval_source("options(warn=2)").unwrap();
    assert_eq!(
        frontend.eval_source("formals(list())"),
        Err(EvalError::Domain(
            "(converted from warning) argument is not a function".into()
        ))
    );
    assert_eq!(
        frontend.eval_source(".Internal(formals())"),
        Err(EvalError::Arity(
            "0 arguments passed to .Internal(formals) which requires 1".into()
        ))
    );
    assert_eq!(
        frontend.eval_source(".Internal(formals(sum,{cat('EXTRA');1L}))"),
        Err(EvalError::Arity(
            "2 arguments passed to .Internal(formals) which requires 1".into()
        ))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("EXTRA".into())]
    );
}
