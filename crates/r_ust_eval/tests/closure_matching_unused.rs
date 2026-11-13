use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn unused_diagnostics_preserve_origin_and_stop_at_first_transfer() {
    for (source, error, output) in [
        ("g<-function()NULL;f<-function(...)g(...);h<-function(){f(unused=return(42));99};h()", None, ""),
        ("g<-function()NULL;h<-function(){g(unused=return(42));99};h()", Some("unused argument (unused = return(42))"), ""),
        ("g<-function()NULL;f<-function(...)g(...);h<-function(){for(i in 1:2){f(unused=break);print(99)};42};h()", None, ""),
        ("g<-function()NULL;f<-function(...)g(...);h<-function(){for(i in 1:2){f(unused=next);print(99)};42};h()", None, ""),
        ("g<-function()NULL;f<-function(...)g(...);f(unused=stop('forced unused'))", Some("forced unused"), ""),
        ("g<-function()NULL;f<-function(...)g(...);f(unused=print(9))", Some("unused argument (unused = 9)"), "[1] 9\n"),
        ("g<-function()NULL;g(unused=print(9))", Some("unused argument (unused = print(9))"), ""),
        ("g<-function()NULL;f<-function(...)g(...);h<-function(){f(first=return(42),second=print(99));100};h()", None, ""),
    ] {
        let mut frontend = Frontend::default();
        let result = frontend.eval_source(source);
        match error {
            Some(expected) => match result.unwrap_err() {
                EvalError::Stop { message, call_requested } if expected == "forced unused" => {
                    assert_eq!(message, expected, "{source}");
                    assert!(call_requested, "{source}");
                }
                EvalError::Arity(actual) => assert_eq!(actual, expected, "{source}"),
                other => panic!("unexpected condition for {source}: {other:?}"),
            },
            None => assert_eq!(result.unwrap().last(), Some(&Sexp::Real(vec![42.0])), "{source}"),
        }
        let expected = if output.is_empty() { vec![] } else { vec![OutputEvent::Stdout(output.into())] };
        assert_eq!(frontend.runtime_mut().take_output(), expected, "{source}");
    }
}

#[test]
fn unused_rendering_reuses_cache_and_preserves_source_expressions() {
    for (source, expected, output) in [
        (
            "g<-function()NULL;f<-function(...) {list(...);g(...)};f(x={cat('ONCE');9L})",
            "unused argument (x = 9)",
            "ONCE",
        ),
        (
            "g<-function()NULL;f<-function(x)g(unused=x);f({cat('NO');9L})",
            "unused argument (unused = x)",
            "",
        ),
        (
            "g<-function()NULL;g(a=,b=1L)",
            "unused arguments (alist(a = , b = 1))",
            "",
        ),
        (
            "g<-function()NULL;g(,1L)",
            "unused arguments (alist(, 1))",
            "",
        ),
        (
            "g<-function()NULL;g(x={cat('LINE\\n');9L})",
            "unused argument (x = {\n    cat(\"LINE\\n\")\n    9\n})",
            "",
        ),
        (
            "g<-function()NULL;g(`a b`=1L)",
            "unused argument (`a b` = 1)",
            "",
        ),
        (
            "g<-function()NULL;g(x=return(4L))",
            "unused argument (x = return(4))",
            "",
        ),
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(source),
            Err(EvalError::Arity(expected.into())),
            "{source}"
        );
        let expected = if output.is_empty() {
            vec![]
        } else {
            vec![OutputEvent::Stdout(output.into())]
        };
        assert_eq!(frontend.runtime_mut().take_output(), expected, "{source}");
    }
}
