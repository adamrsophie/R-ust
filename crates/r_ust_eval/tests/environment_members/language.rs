use super::*;
use r_ust_sexp::{canonical_language, CallArgument, SexpType};

#[test]
fn quoted_member_assignment_stays_inert_and_round_trips() {
    let mut frontend = Frontend::default();
    for source in ["h[[1L]]$x <- {cat('NEVER');9L}", "h$child[['x']] = 7L"] {
        let quoted = frontend
            .eval_source(&format!("quote({source})"))
            .unwrap()
            .remove(0);
        assert_eq!(quoted, canonical_language(&expression(source)).unwrap());
        assert_eq!(quoted.sexptype(), SexpType::Language);
        assert!(frontend.runtime_mut().take_output().is_empty());
        assert_eq!(frontend.runtime().get("h").unwrap(), None);
        let printed = r_ust_eval::format_sexp(&quoted).unwrap();
        assert_eq!(canonical_language(&expression(&printed)).unwrap(), quoted);
    }
    let formal = frontend
        .eval_source("formals(function(x=h[[1L]]$x<-9L)NULL)")
        .unwrap()
        .remove(0);
    let expected = r_ust_sexp::ListElement {
        name: Some("x".into()),
        value: canonical_language(&expression("h[[1L]]$x<-9L")).unwrap(),
    };
    assert_eq!(formal, Sexp::pairlist(vec![expected]));
    assert_eq!(
        frontend
            .runtime_mut()
            .eval(&Sexp::Call {
                function: Box::new(Sexp::Symbol("typeof".into())),
                arguments: vec![CallArgument::positional(expression("quote(h[[1L]]$x<-9L)"))],
            })
            .unwrap(),
        Sexp::String(vec![Some("language".into())])
    );
}
