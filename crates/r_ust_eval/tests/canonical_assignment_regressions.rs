use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{ClosureEnvironment, Sexp, SyntaxPrimitive};

fn truth(frontend: &mut Frontend, source: &str) {
    assert_eq!(
        frontend.eval_source(source).unwrap().last(),
        Some(&Sexp::Logical(vec![Some(true)])),
        "{source}"
    );
}

#[test]
fn inherited_root_is_local_before_selector_failure() {
    let mut frontend = Frontend::default();
    frontend.eval_source("e<-new.env();e$x<-1L;holder<-list(e);f<-function(){kept<<-function()holder;holder[[{cat('INDEX');0L}]]$x<-{cat('VALUE');9L}}").unwrap();
    assert!(frontend.eval_source("f()").is_err());
    let Some(Sexp::Closure {
        environment: ClosureEnvironment::Captured(frame),
        ..
    }) = frontend.runtime().get("kept").unwrap()
    else {
        panic!("retained closure missing");
    };
    assert!(frame.contains_local("holder").unwrap());
    truth(
        &mut frontend,
        "identical(kept(),holder) && identical(e$x,1L)",
    );
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("VALUE".into()),
            OutputEvent::Stdout("INDEX".into()),
        ]
    );
}

#[test]
fn rhs_then_outer_getter_inner_setter_outer_setter() {
    let mut frontend = Frontend::default();
    frontend.eval_source("e<-new.env();h<-list(e);h[[{cat('OUTER');1L}]][[{cat('INNER');'x'}]]<-{cat('VALUE');3L}").unwrap();
    truth(&mut frontend, "identical(e$x,3L)");
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [
            OutputEvent::Stdout("VALUE".into()),
            OutputEvent::Stdout("OUTER".into()),
            OutputEvent::Stdout("INNER".into()),
            OutputEvent::Stdout("OUTER".into()),
        ]
    );
}

#[test]
fn absent_root_errors_after_rhs_without_creating_binding() {
    let mut frontend = Frontend::default();
    let error = frontend
        .eval_source("never_defined$x<-{cat('VALUE');1L}")
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("object 'never_defined' not found"),
        "{error}"
    );
    assert_eq!(frontend.runtime().get("never_defined").unwrap(), None);
    assert_eq!(
        frontend.runtime_mut().take_output(),
        [OutputEvent::Stdout("VALUE".into())]
    );
}

#[test]
fn storage_mode_setter_has_identity_arity_and_lexical_override() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend
            .eval_source(".Primitive('storage.mode<-')")
            .unwrap(),
        [Sexp::Primitive(r_ust_sexp::PrimitiveId::Syntax(
            SyntaxPrimitive::StorageModeAssign
        ))]
    );
    let error = frontend
        .eval_source(".Primitive('storage.mode<-')(1L)")
        .unwrap_err();
    assert!(error.to_string().contains("requires 2"), "{error}");
    frontend
        .eval_source("x<-structure(list(a=1L),class='box');saved<-x")
        .unwrap();
    assert!(matches!(
        frontend.eval_source("storage.mode(x)<-'integer'"),
        Err(EvalError::NotImplemented(_))
    ));
    truth(&mut frontend, "identical(x,saved)");
    frontend
        .eval_source("`storage.mode<-`<-function(x,value)list(value);storage.mode(x)<-'custom'")
        .unwrap();
    truth(&mut frontend, "identical(x,list('custom'))");
}

#[test]
fn changing_selector_without_alias_mutates_both_children() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("h<-list(list(x=1L),list(x=2L));i<-0L;h[[{i<-i+1L;i}]]$x<-9L")
        .unwrap();
    truth(
        &mut frontend,
        "identical(i,2L) && identical(h,list(list(x=9L),list(x=9L)))",
    );
}

#[test]
fn changing_selector_preserves_shared_children_and_aliases() {
    for (setup, retained) in [
        (
            "h<-list(list(x=1L),list(x=2L));alias<-h",
            "identical(alias,list(list(x=1L),list(x=2L)))",
        ),
        (
            "child<-list(x=1L);h<-list(child,list(x=2L))",
            "identical(child,list(x=1L))",
        ),
        (
            "h<-list(list(x=1L),list(x=2L));a<-h[[1L]]",
            "identical(a,list(x=1L))",
        ),
    ] {
        let mut frontend = Frontend::default();
        frontend.eval_source(setup).unwrap();
        frontend
            .eval_source("i<-0L;h[[{i<-i+1L;i}]]$x<-9L")
            .unwrap();
        truth(
            &mut frontend,
            "identical(i,2L) && identical(h,list(list(x=1L),list(x=9L)))",
        );
        truth(&mut frontend, retained);
    }
}

#[test]
fn inherited_and_stable_selector_roots_preserve_aliases() {
    let mut frontend = Frontend::default();
    frontend.eval_source("h<-list(list(x=1L),list(x=2L));f<-function(){i<-0L;h[[{i<-i+1L;i}]]$x<-9L;h};local_result<-f()").unwrap();
    truth(&mut frontend, "identical(local_result,list(list(x=1L),list(x=9L))) && identical(h,list(list(x=1L),list(x=2L)))");
    frontend.eval_source("alias<-h;h[[1L]]$x<-9L").unwrap();
    truth(
        &mut frontend,
        "identical(h,list(list(x=9L),list(x=2L))) && identical(alias,list(list(x=1L),list(x=2L)))",
    );
}
