use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;

#[test]
fn quoted_names_reach_bindings_arguments_and_members() {
    for (source, expected) in [
        ("`my var`<-3;`my var`+1", 4.0),
        ("`if`<-7;`if`", 7.0),
        ("`TRUE`<-7;`TRUE`", 7.0),
        ("f<-function(`if`,`a b`=2)`if`+`a b`;f(`if`=1,`a b`=4)", 5.0),
        ("f<-function(`x y`=3)`x y`;f(`x y`=)", 3.0),
        ("x<-list(`a b`=11);x$`a b`<-12;x$`a b`", 12.0),
        ("`f name`<-function(`x y`)`x y`;`f name`(4)", 4.0),
        ("for(`loop var` in 1:3)NULL;`loop var`+0", 3.0),
        ("`a b`<-1;f<-function(){`a b`<<-4};f();`a b`", 4.0),
    ] {
        assert_eq!(
            Frontend::default().eval_source(source).unwrap().last(),
            Some(&Sexp::Real(vec![expected])),
            "{source}"
        );
    }
}

#[test]
fn dots_are_not_ordinary_values_even_when_backtick_quoted() {
    for source in ["...", "`...`", "`...`<-7;`...`", "(function(...)`...`)(1)"] {
        assert!(
            matches!(
                Frontend::default().eval_source(source),
                Err(EvalError::Domain(_))
            ),
            "{source}"
        );
    }
    assert_eq!(
        Frontend::default()
            .eval_source("(function(...)sum(`...`))(1,2)")
            .unwrap(),
        vec![Sexp::Real(vec![3.0])]
    );
}
