#[path = "body_inspection/sequence.rs"]
mod sequence;

use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_parse::Parser;
use r_ust_sexp::{canonical_language, Sexp};

#[test]
fn body_explicit_preserves_canonical_source_without_forcing() {
    let mut frontend = Frontend::default();
    for source in [
        "stop('UNFORCED')",
        "{ stop('UNFORCED'); x + 1L }",
        "function(y = stop('UNFORCED')) y",
        "if (x) y else z",
    ] {
        let parsed = Parser.parse_all(source).unwrap();
        let expected = canonical_language(&parsed.expressions[0]).unwrap();
        let input = format!(
            "f <- function(x = stop('UNFORCED'), ...) {source}; b <- body; b(f); (identity(b))(f)"
        );
        let values = frontend.eval_source(&input).unwrap();
        assert_eq!(&values[2..], &[expected.clone(), expected], "{source}");
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
}

#[test]
fn body_primitive_and_nonfunction_conditions() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend
            .eval_source("body(sum); body(.Primitive('quote'))")
            .unwrap(),
        [Sexp::Nil, Sexp::Nil]
    );
    assert!(frontend.runtime_mut().take_output().is_empty());
    frontend.eval_source("options(warn=1)").unwrap();
    for source in ["body(1L)", "body(NULL)", "body(list())"] {
        assert_eq!(frontend.eval_source(source).unwrap(), [Sexp::Nil]);
        assert_eq!(
            frontend.runtime_mut().take_output(),
            [OutputEvent::Stderr(
                "Warning: argument is not a function\n".into()
            )]
        );
    }
    frontend.eval_source("options(warn=2)").unwrap();
    assert_eq!(
        frontend.eval_source("body(1L)"),
        Err(EvalError::Domain(
            "(converted from warning) argument is not a function".into()
        ))
    );
    assert_eq!(
        frontend.eval_source("body('identity')"),
        Err(EvalError::UndefinedFunction("get".into()))
    );
    assert_eq!(
        frontend.eval_source(".Internal(body())"),
        Err(EvalError::Arity(
            "0 arguments passed to .Internal(body) which requires 1".into()
        ))
    );
    assert_eq!(
        frontend.eval_source(".Internal(body(sum,{cat('EXTRA');1L}))"),
        Err(EvalError::Arity(
            "2 arguments passed to .Internal(body) which requires 1".into()
        ))
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("EXTRA".into())]
    );
}

#[test]
fn body_wrapper_retains_genuine_source_and_default() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source_printed("formals(body);body(body)")
        .unwrap();
    assert_eq!(frontend.runtime_mut().take_output(), [
        OutputEvent::Stdout("$fun\nsys.function(sys.parent())\n\n".into()),
        OutputEvent::Stdout("{\n    if (is.character(fun)) \n        fun <- get(fun, mode = \"function\", envir = parent.frame())\n    .Internal(body(fun))\n}\n".into()),
    ]);
}

#[test]
fn body_default_at_top_level_returns_its_own_source() {
    let mut frontend = Frontend::default();
    frontend.eval_source_printed("body()").unwrap();
    assert_eq!(frontend.runtime_mut().take_output(), [OutputEvent::Stdout(
        "{\n    if (is.character(fun)) \n        fun <- get(fun, mode = \"function\", envir = parent.frame())\n    .Internal(body(fun))\n}\n".into()
    )]);
}
