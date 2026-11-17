use r_ust_eval::{Frontend, OutputEvent};
use r_ust_sexp::Sexp;

#[test]
fn complex_superassignment_skips_local_root_and_uses_caller_selectors() {
    for source in [
        "x<-list(a=1L);f<-function(){x<-list(a=2L);x$a<<-3L;x$a};identical(f(),2L) && identical(x,list(a=3L))",
        "x<-list(1L,2L,3L);i<-0L;f<-function(){i<-2L;x[[{i<-i+1L;i}]]<<-9L;i};identical(f(),3L) && identical(i,0L) && identical(x,list(1L,2L,9L))",
        "x<-list(list(a=1L));saved<-x;f<-function(){x[[1L]]$a<<-7L};f();identical(x,list(list(a=7L))) && identical(saved,list(list(a=1L)))",
    ] {
        let mut frontend = Frontend::default();
        assert_eq!(frontend.eval_source(source).unwrap().last(),Some(&Sexp::Logical(vec![Some(true)])),"{source}");
        assert!(frontend.runtime_mut().take_output().is_empty());
    }
}

#[test]
fn invalid_computed_assignment_is_a_catchable_runtime_condition() {
    let mut frontend = Frontend::default();
    assert_eq!(frontend.eval_source("tryCatch(parent.frame()$X<-{cat('RHS');list()},error=function(e)conditionMessage(e))").unwrap().last(),
        Some(&Sexp::String(vec![Some("invalid (NULL) left side of assignment".into())])));
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("RHS".into())]
    );
}
