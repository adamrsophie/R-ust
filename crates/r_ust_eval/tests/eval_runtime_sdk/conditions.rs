use super::{drained, integer, last};
use r_ust_eval::Frontend;
use r_ust_sexp::Sexp;

fn strings(values: &[&str]) -> Sexp {
    Sexp::String(values.iter().map(|value| Some((*value).into())).collect())
}

#[test]
fn eval_faults_preserve_actual_source_calls_and_condition_fields() {
    for (source, message) in [
        (
            "evalq(1L, globalenv(), 2L)",
            "invalid 'enclos' argument of type 'NULL'",
        ),
        (
            "eval(1L, TRUE)",
            "invalid 'envir' argument of type 'logical'",
        ),
        (
            "eval(1L, c(0L,1L))",
            "numeric 'envir' arg not of length one",
        ),
        (
            "eval(1L, NA_integer_)",
            "invalid 'envir' argument of type 'integer'",
        ),
        ("eval(1L, 999L)", "not that many frames on the stack"),
    ] {
        let mut frontend = Frontend::default();
        let program = format!("caught<-tryCatch({source},error=function(e)e);identical(conditionCall(caught),quote({source}))");
        assert_eq!(
            last(&mut frontend, &program),
            Sexp::Logical(vec![Some(true)]),
            "{source}"
        );
        assert_eq!(
            last(&mut frontend, "class(caught)"),
            strings(&["simpleError", "error", "condition"]),
            "{source}"
        );
        assert_eq!(
            last(&mut frontend, "conditionMessage(caught)"),
            strings(&[message]),
            "{source}"
        );
        drained(&mut frontend);
    }
}

#[test]
fn missing_condition_subclasses_include_bytecode_lookup_requirement() {
    // Pinned R hidden-reference captures these subclasses. Keep the getvarError
    // assertion active: interpreted base closures alone cannot satisfy it.
    for (source, class, message) in [
        (
            "eval()",
            "getvarError",
            "argument \"expr\" is missing, with no default",
        ),
        (
            "evalq()",
            "evalError",
            "argument is missing, with no default",
        ),
        (
            "eval(quote(expr=))",
            "evalError",
            "argument is missing, with no default",
        ),
    ] {
        let mut frontend = Frontend::default();
        let program = format!("caught<-tryCatch({source},error=function(e)e);identical(conditionCall(caught),quote({source}))");
        assert_eq!(
            last(&mut frontend, &program),
            Sexp::Logical(vec![Some(true)]),
            "{source}"
        );
        let class = if class == "getvarError" && !frontend.runtime().bytecode_enabled() {
            "evalError"
        } else {
            class
        };
        assert_eq!(
            last(&mut frontend, "class(caught)"),
            strings(&[class, "missingArgError", "error", "condition"]),
            "{source}"
        );
        assert_eq!(
            last(&mut frontend, "conditionMessage(caught)"),
            strings(&[message]),
            "{source}"
        );
    }
}

#[test]
fn named_list_environment_keeps_na_empty_and_first_duplicate_semantics() {
    for (source, expected) in [
        ("evalq(`NA`, structure(list(41L),names=NA_character_))", 41),
        (
            "evalq(x,structure(list(42L,43L,44L),names=c('', 'x','x')))",
            43,
        ),
        ("evalq(x,pairlist(x=45L,x=46L))", 45),
        ("x<-47L;evalq(x,list(48L))", 47),
    ] {
        assert_eq!(
            last(&mut Frontend::default(), source),
            integer(expected),
            "{source}"
        );
    }
}

#[test]
fn assign_inherits_updates_selected_chain_and_unforced_immediate_is_invisible() {
    let mut frontend = Frontend::default();
    last(&mut frontend, "p<-new.env();p$x<-49L;e<-new.env(parent=p)");
    assert_eq!(
        last(&mut frontend, "assign('x',50L,envir=e,inherits=TRUE)"),
        integer(50)
    );
    assert!(!frontend.runtime().is_visible());
    assert_eq!(last(&mut frontend, "p$x"), integer(50));
    assert_eq!(
        last(&mut frontend, "is.null(e$x)"),
        Sexp::Logical(vec![Some(true)])
    );
    assert_eq!(
        last(
            &mut frontend,
            "assign('y',51L,envir=e,immediate=stop('UNREACHED'))"
        ),
        integer(51)
    );
    assert!(!frontend.runtime().is_visible());
    assert_eq!(last(&mut frontend, "e$y"), integer(51));
    drained(&mut frontend);
}
