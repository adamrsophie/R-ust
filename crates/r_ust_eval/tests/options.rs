use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{ListElement, Sexp};

fn text(value: &str) -> Sexp {
    Sexp::String(vec![Some(value.to_string())])
}

fn last(frontend: &mut Frontend, code: &str) -> Sexp {
    frontend.eval_source(code).unwrap().pop().unwrap()
}

#[test]
fn options_are_runtime_owned_and_return_previous_values() {
    let mut first = Frontend::default();
    let mut second = Frontend::default();
    assert_eq!(last(&mut first, "getOption('prompt')"), text("> "));
    assert_eq!(
        last(&mut first, "options(answer=1,answer=2)"),
        Sexp::List(vec![
            ListElement {
                name: Some("answer".into()),
                value: Sexp::Nil
            },
            ListElement {
                name: Some("answer".into()),
                value: Sexp::Real(vec![1.0])
            },
        ])
    );
    assert!(!first.runtime().is_visible());
    assert_eq!(
        last(&mut first, "getOption('answer')"),
        Sexp::Real(vec![2.0])
    );
    assert_eq!(last(&mut second, "getOption('answer')"), Sexp::Nil);
    assert_eq!(
        last(
            &mut first,
            "old<-options(answer=3,other=list(x=4));options(old);getOption('answer')"
        ),
        Sexp::Real(vec![2.0])
    );
    assert_eq!(last(&mut first, "getOption('other')"), Sexp::Nil);
    assert_eq!(
        last(&mut first, "options(answer=NULL);getOption('answer',9)"),
        Sexp::Real(vec![9.0])
    );
}

#[test]
fn get_option_matches_arguments_before_forcing_and_keeps_default_lazy() {
    let mut frontend = Frontend::default();
    assert_eq!(
        last(&mut frontend, "getOption('width',notDefined)"),
        Sexp::Integer(vec![Some(80)])
    );
    assert_eq!(
        last(&mut frontend, "getOption(def=9,x='absent')"),
        Sexp::Real(vec![9.0])
    );
    assert_eq!(
        last(
            &mut frontend,
            "(function(...)getOption(...))('width',notDefined)"
        ),
        Sexp::Integer(vec![Some(80)])
    );
    assert_eq!(
        last(
            &mut frontend,
            "withVisible(getOption('absent',invisible(9)))$visible"
        ),
        Sexp::Logical(vec![Some(false)])
    );
    assert_eq!(
        last(
            &mut frontend,
            "withVisible(getOption(invisible('width')))$visible"
        ),
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(
        last(&mut frontend, "(function()getOption('absent',return(7)))()"),
        Sexp::Real(vec![7.0])
    );
    for code in [
        "getOption()",
        "getOption(1)",
        "getOption(as.character(NULL))",
        "getOption(c('width','digits'))",
        "getOption('')",
        "getOption(bogus=print(9))",
        "getOption('width',1,print(9))",
        "getOption(x='width',x=print(9))",
    ] {
        let error = frontend.eval_source(code).unwrap_err();
        assert!(
            !matches!(error, EvalError::UndefinedSymbol(_)),
            "{code}: {error}"
        );
        assert!(frontend.runtime_mut().take_output().is_empty(), "{code}");
    }
}

#[test]
fn options_support_queries_lists_visibility_and_independent_snapshots() {
    let mut frontend = Frontend::default();
    last(
        &mut frontend,
        "options(list(z=3,a=1));saved<-options();options(a=2)",
    );
    assert_eq!(last(&mut frontend, "saved$a"), Sexp::Real(vec![1.0]));
    assert_eq!(
        last(&mut frontend, "options('width')$width"),
        Sexp::Integer(vec![Some(80)])
    );
    assert_eq!(
        last(&mut frontend, "options(c('width','digits'))$width"),
        Sexp::Integer(vec![Some(80)])
    );
    assert_eq!(
        last(&mut frontend, "withVisible(options())$visible"),
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(
        last(&mut frontend, "withVisible(options(NULL))$visible"),
        Sexp::Logical(vec![Some(false)])
    );
    assert_eq!(
        last(&mut frontend, "withVisible(options(list()))$visible"),
        Sexp::Logical(vec![Some(false)])
    );
    assert_eq!(
        last(&mut frontend, "withVisible(options(a=3,'width'))$visible"),
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(
        last(&mut frontend, "options(list('width',a=4))$width"),
        Sexp::Integer(vec![Some(80)])
    );
    for code in [
        "options(1)",
        "options(list(1))",
        "options(as.character(NULL))",
        "options('')",
    ] {
        assert!(frontend.eval_source(code).is_err(), "{code}");
    }
}

#[test]
fn setters_coerce_validate_and_preserve_earlier_changes_on_failure() {
    let mut frontend = Frontend::default();
    for (code, expected) in [
        (
            "options(width=79.9);getOption('width')",
            Sexp::Integer(vec![Some(79)]),
        ),
        (
            "options(prompt=c('ask: ','ignored'));getOption('prompt')",
            text("ask: "),
        ),
        ("options(prompt=42);getOption('prompt')", text("42")),
        ("options(prompt=NA_real_);getOption('prompt')", text("NA")),
        (
            "options(check.bounds=NA);getOption('check.bounds')",
            Sexp::Logical(vec![None]),
        ),
        (
            "options(keep.parse.data=3);getOption('keep.parse.data')",
            Sexp::Real(vec![3.0]),
        ),
        (
            "options(warn=TRUE);getOption('warn')",
            Sexp::Integer(vec![Some(1)]),
        ),
    ] {
        assert_eq!(last(&mut frontend, code), expected, "{code}");
    }
    for code in [
        "options(width=9)",
        "options(width=10001)",
        "options(digits=23)",
        "options(expressions=24)",
        "options(prompt='')",
        "options(prompt=NA_character_)",
        "options(continue=NULL)",
        "options(width=NULL)",
        "options(quiet=1)",
        "options(quiet=NA)",
        "options(warn=NA)",
        "options(warn=c(1,2))",
        "options(showNCalls=c(30,31))",
        "options(matprod='wrong')",
        "options(par.ask.default=TRUE)",
    ] {
        assert!(frontend.eval_source(code).is_err(), "{code}");
    }
    assert!(frontend.eval_source("options(partial=5,width=2)").is_err());
    assert_eq!(
        last(&mut frontend, "getOption('partial')"),
        Sexp::Real(vec![5.0])
    );
    assert_eq!(
        last(&mut frontend, "getOption('width')"),
        Sexp::Integer(vec![Some(79)])
    );
}

#[test]
fn option_coercion_and_deprecation_warnings_retain_r_values() {
    let mut frontend = Frontend::default();
    assert_eq!(
        last(
            &mut frontend,
            "options(deparse.cutoff='bad');getOption('deparse.cutoff')"
        ),
        Sexp::Integer(vec![None])
    );
    assert_eq!(frontend.runtime_mut().take_output().len(), 1);
    last(
        &mut frontend,
        "options(stringsAsFactors=TRUE);options(stringsAsFactors=TRUE)",
    );
    assert_eq!(frontend.runtime_mut().take_output().len(), 1);
    assert_eq!(
        last(
            &mut frontend,
            "options(PCRE_study=-1);getOption('PCRE_study')"
        ),
        Sexp::Logical(vec![Some(true)])
    );
    assert!(frontend.runtime_mut().take_output().is_empty());
    assert_eq!(
        last(&mut frontend, "options(scipen=-10);getOption('scipen')"),
        Sexp::Integer(vec![Some(-9)])
    );
    assert!(matches!(
        frontend.runtime_mut().take_output().as_slice(),
        [OutputEvent::Stderr(_)]
    ));
}
