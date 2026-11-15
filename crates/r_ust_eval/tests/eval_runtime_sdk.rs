use r_ust_eval::Frontend;
use r_ust_sexp::{CallArgument, Environment, ListElement, PrimitiveId, Sexp};

#[path = "eval_runtime_sdk/conditions.rs"]
mod conditions;
#[path = "eval_runtime_sdk/lifetime.rs"]
mod lifetime;
#[path = "eval_runtime_sdk/promises.rs"]
mod promises;

fn integer(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}

fn last(frontend: &mut Frontend, source: &str) -> Sexp {
    frontend.eval_source(source).unwrap().pop().unwrap()
}

fn global(frontend: &mut Frontend) -> Environment {
    let Sexp::Environment(frame) = last(frontend, "globalenv()") else {
        panic!("globalenv must return an environment")
    };
    frame
}

fn transported(value: Sexp) -> Sexp {
    Sexp::Extract {
        value: Box::new(Sexp::List(vec![ListElement { name: None, value }])),
        index: Box::new(integer(1)),
    }
}

fn evaluate(payload: Sexp, environment: &Environment) -> Sexp {
    Sexp::Call {
        function: Box::new(Sexp::Primitive(PrimitiveId::InternalEval)),
        arguments: vec![
            CallArgument::positional(transported(payload)),
            CallArgument::positional(Sexp::Environment(environment.clone())),
            CallArgument::positional(Sexp::Nil),
        ],
    }
}

fn drained(frontend: &mut Frontend) {
    assert_eq!(last(frontend, "sys.nframe()"), integer(0));
    assert_eq!(last(frontend, "42L"), integer(42));
    assert!(frontend.runtime().is_visible());
}

#[test]
fn context_exposes_callable_internal_identity_without_public_registration() {
    let mut frontend = Frontend::default();
    let callable = last(&mut frontend, "evalq(sys.function())");
    assert_eq!(callable, Sexp::Primitive(PrimitiveId::InternalEval));
    assert_eq!(
        r_ust_eval::format_sexp(&callable).unwrap(),
        ".Primitive(\"eval\")"
    );
    assert_eq!(PrimitiveId::from_name("eval"), None);
    assert_eq!(
        last(&mut frontend, "typeof(evalq(sys.function()))"),
        Sexp::String(vec![Some("builtin".into())])
    );
    assert_eq!(
        last(&mut frontend, "formals(evalq(sys.function()))"),
        Sexp::Nil
    );
    assert_eq!(
        last(&mut frontend, "identical(evalq(sys.function()),eval)"),
        Sexp::Logical(vec![Some(false)])
    );
    last(&mut frontend, "f<-evalq(sys.function())");
    assert_eq!(
        last(
            &mut frontend,
            "f(quote(invisible(43L)),globalenv(),baseenv())"
        ),
        integer(43)
    );
    assert!(!frontend.runtime().is_visible());
    assert!(frontend.eval_source(".Primitive('eval')").is_err());
    drained(&mut frontend);
}

#[test]
fn raw_numeric_frames_select_the_live_wrapper_once() {
    let mut frontend = Frontend::default();
    for source in [
        "(function(){x<-19L;evalq(x,-1)})()",
        "(function(){x<-19L;(function()evalq(x,1L))()})()",
        "(function(){x<-19L;evalq(evalq(x,-3))})()",
    ] {
        assert_eq!(last(&mut frontend, source), integer(19), "{source}");
        drained(&mut frontend);
    }
}

#[test]
fn selected_loop_and_return_boundaries_preserve_visibility_and_stop_sequences() {
    let mut frontend = Frontend::default();
    for source in [
        "for(i in 1:2)evalq(break);42L",
        "for(i in 1:2){evalq(next);stop('UNREACHED')};42L",
        "eval(expression(return(42L),stop('UNREACHED')))",
    ] {
        assert_eq!(last(&mut frontend, source), integer(42), "{source}");
        drained(&mut frontend);
    }
    assert_eq!(
        last(
            &mut frontend,
            "eval(expression(return(invisible(42L)),stop('UNREACHED')))"
        ),
        integer(42)
    );
    assert!(!frontend.runtime().is_visible());
    assert!(frontend.runtime_mut().take_output().is_empty());
    drained(&mut frontend);
}
