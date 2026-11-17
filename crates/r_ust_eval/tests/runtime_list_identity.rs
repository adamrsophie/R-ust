use r_ust_eval::Frontend;
use r_ust_sexp::{ListElement, ListObject, Sexp};

#[test]
fn runtime_lists_expose_snapshots_and_explicit_sdk_lists_keep_identity() {
    let mut frontend = Frontend::default();
    let Sexp::Environment(owner) = frontend.eval_source("globalenv()").unwrap().pop().unwrap()
    else {
        panic!("global environment")
    };
    let value = Sexp::List(vec![ListElement {
        name: None,
        value: Sexp::Integer(vec![Some(1)]),
    }]);
    let explicit = ListObject::from_value(&owner, &value).unwrap();
    assert_eq!(
        frontend
            .runtime_mut()
            .eval(&Sexp::ListObject(explicit.clone()))
            .unwrap(),
        Sexp::ListObject(explicit)
    );
    assert_eq!(
        frontend.eval_source("x<-list(1L);x").unwrap().last(),
        Some(&value)
    );
    assert_eq!(frontend.runtime().get("x").unwrap(), Some(value));
}

#[test]
fn replacement_rhs_aliases_cannot_create_cycles_or_mutate_retained_values() {
    for source in [
        "x<-list();x[[1L]]<-x;identical(x,list(list()))",
        "x<-list(list(a=1L));x[[1L]]$b<-x[[1L]];identical(x,list(list(a=1L,b=list(a=1L))))",
        "x<-list(list(a=1L));saved<-x[[1L]];x[[1L]]$a<-9L;identical(saved,list(a=1L)) && identical(x,list(list(a=9L)))",
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(frontend.eval_source(source).unwrap().last(),Some(&Sexp::Logical(vec![Some(true)])),"{source}");
    }
}

#[test]
fn selector_aliases_match_r_at_each_replacement_stage() {
    for source in include_str!("fixtures/replacement-aliases-r-4.6.0.txt").lines() {
        let mut frontend = Frontend::default();
        assert_eq!(
            frontend.eval_source(source).unwrap().last(),
            Some(&Sexp::Logical(vec![Some(true)])),
            "{source}"
        );
    }
}
