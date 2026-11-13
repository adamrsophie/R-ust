use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn internal_vector_checks_syntax_and_validation_in_r_order() {
    for (source, expected, output) in [
        (
            ".Internal()",
            "0 arguments passed to '.Internal' which requires 1",
            "",
        ),
        (
            ".Internal(vector('integer',2),cat('NO'))",
            "2 arguments passed to '.Internal' which requires 1",
            "",
        ),
        (".Internal(1)", "invalid .Internal() argument", ""),
        (".Internal(vector)", "invalid .Internal() argument", ""),
        (
            ".Internal((vector('integer',2)))",
            "there is no .Internal function '('",
            "",
        ),
        (
            ".Internal((function() 1)())",
            "invalid .Internal() argument",
            "",
        ),
        (
            ".Internal(print(cat('NO')))",
            "there is no .Internal function 'print'",
            "",
        ),
        (
            ".Internal(vector({cat('A');'integer'},2,{cat('B');3}))",
            "3 arguments passed to .Internal(vector) which requires 2",
            "AB",
        ),
        (
            ".Internal(vector({cat('A');'integer'},,{cat('NO');3}))",
            "argument 2 is empty",
            "A",
        ),
        (
            "vector(list(list('integer')),0)",
            "vector: cannot make a vector of mode 'list(\"integer\")'.",
            "",
        ),
        (
            "vector('bad',4503599627370496)",
            "vector: cannot make a vector of mode 'bad'.",
            "",
        ),
        (
            "vector('bad',4503599627370497)",
            "vector size specified is too large",
            "",
        ),
        (
            "vector(factor('integer'),4503599627370497)",
            "vector size specified is too large",
            "",
        ),
        ("vector(stop('mode'),NA_integer_)", "mode", ""),
        ("vector('bad',stop('length'))", "length", ""),
        (
            "vector({cat('M');'bad'},{cat('L');-1})",
            "invalid 'length' argument",
            "ML",
        ),
        (
            "options(warn=1);.Internal(vector({cat('M');'integer'},{cat('L');'bad'}))",
            "vector size cannot be NA/NaN",
            "MLstderr:Warning: NAs introduced by coercion\n",
        ),
    ] {
        let mut frontend = Frontend::default();
        let error = frontend.eval_source(source).unwrap_err();
        let message = match error {
            EvalError::Arity(message)
            | EvalError::Domain(message)
            | EvalError::Stop { message, .. } => message,
            other => panic!("{source}: unexpected error {other:?}"),
        };
        assert_eq!(message, expected, "{source}");
        let actual: String = frontend
            .runtime_mut()
            .take_output()
            .into_iter()
            .map(|event| match event {
                OutputEvent::Stdout(text) => text,
                OutputEvent::Stderr(text) => format!("stderr:{text}"),
                other @ OutputEvent::StdoutBytes(_) => {
                    panic!("{source}: unexpected event {other:?}")
                }
            })
            .collect();
        assert_eq!(actual, output, "{source}");
        println!("PASS\t{source}\t{message}\t{actual}");
    }
}

#[test]
fn internal_vector_uses_registry_and_positional_arguments() {
    for source in [
        ".Internal(vector(a='integer',a=2))",
        "vector<-function(...)stop('NO');.Internal(vector('integer',2))",
        ".Internal<-1;.Internal(vector('integer',2))",
        "integer<-1;integer(2)",
        "vector(matrix('integer',1),2)",
    ] {
        let values = Frontend::default().eval_source(source).unwrap();
        assert_eq!(
            values.last(),
            Some(&Sexp::Integer(vec![Some(0), Some(0)])),
            "{source}"
        );
        println!("PASS\t{source}");
    }
}

#[test]
fn unrepresented_vector_modes_report_storage_limit_after_length_validation() {
    for mode in ["complex", "raw", "expression", "pairlist"] {
        for length in ["0", "2", "-0.9"] {
            let source = format!("vector('{mode}',{length})");
            assert_eq!(
                Frontend::default().eval_source(&source),
                Err(EvalError::NotImplemented(
                    "complex/raw/expression/pairlist vector storage"
                )),
                "{source}"
            );
        }
        for (length, message) in [
            ("NA_integer_", "vector size cannot be NA"),
            ("list(2)", "invalid 'length' argument"),
        ] {
            let source = format!("vector('{mode}',{length})");
            assert_eq!(
                Frontend::default().eval_source(&source),
                Err(EvalError::Domain(message.into())),
                "{source}"
            );
        }
        println!("LIMIT\t{mode}\tstorage unrepresented; length validation verified");
    }
}

#[test]
fn internal_stop_coerces_factor_codes_without_changing_public_character_dispatch() {
    for (source, expected) in [("factor('a')", "1"), ("list(factor('a'))", "1")] {
        assert_eq!(
            Frontend::default().eval_source(&format!(".Internal(stop(FALSE,{source}))")),
            Err(EvalError::Stop {
                message: expected.into(),
                call_requested: false
            }),
            "{source}"
        );
    }
    assert_eq!(
        Frontend::default().eval_source("stop(factor('a'))"),
        Err(EvalError::Stop {
            message: "a".into(),
            call_requested: true
        })
    );
    assert_eq!(
        Frontend::default().eval_source("as.character(factor('a'))"),
        Ok(vec![Sexp::String(vec![Some("a".into())])])
    );
    println!("PASS\tinternal stop factor codes; public stop and as.character labels");
}

#[test]
fn base_constructor_closures_preserve_identity_and_result_visibility() {
    for source in [
        "identical(double,numeric)",
        "withVisible(integer(invisible(2)))$visible",
        "withVisible(vector('integer',invisible(2)))$visible",
        "withVisible(.Internal(vector('integer',invisible(2))))$visible",
        "withVisible(inherits(invisible(1),'numeric'))$visible",
        "withVisible(inherits(1,'numeric',invisible(FALSE)))$visible",
        "withVisible(.Internal(gettext(NULL,'x',invisible(TRUE))))$visible",
        "withVisible(.Internal(gettext(NULL,invisible(character(0)))))$visible",
        "withVisible(.makeMessage(invisible('x'),domain=invisible(NULL)))$visible",
    ] {
        assert_eq!(
            Frontend::default().eval_source(source),
            Ok(vec![Sexp::Logical(vec![Some(true)])]),
            "{source}"
        );
        println!("PASS\t{source}");
    }
}
