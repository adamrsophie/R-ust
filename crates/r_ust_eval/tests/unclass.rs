use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{AssignmentOperator, ClosureEnvironment, ListElement, Sexp};
#[path = "unclass/dependencies.rs"]
mod dependencies;
#[path = "unclass/sdk.rs"]
mod sdk;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn bind(runtime: &mut RuntimeState, name: &str, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: name.into(),
        value: Box::new(value),
    })?;
    Ok(())
}
fn truth(frontend: &mut Frontend, source: &str) -> Result {
    assert_eq!(
        frontend.eval_source(source)?.last(),
        Some(&Sexp::Logical(vec![Some(true)]))
    );
    Ok(())
}
fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
fn strings(values: &[&str]) -> Sexp {
    Sexp::String(values.iter().map(|s| Some((*s).into())).collect())
}

#[test]
fn unclass_has_real_builtin_identity_and_accepts_first_class_calls() -> Result {
    let mut f = Frontend::default();
    truth(
        &mut f,
        "typeof(unclass)=='builtin' && identical(unclass,.Primitive('unclass'))",
    )?;
    truth(&mut f, "p<-.Primitive('unclass');identical(p(x=1L),1L)")?;
    truth(&mut f, "withVisible(unclass(1L))$visible")
}

#[test]
fn unclass_removes_only_class_and_preserves_missing_blank_and_empty_names() -> Result {
    let mut f = Frontend::default();
    for source in [
        "x<-structure(list(1L,2L,3L),names=c('a','',NA_character_),class=c('box','other'));y<-x;u<-unclass(x);identical(names(u),c('a','',NA_character_)) && !is.object(u) && identical(x,y)",
        "x<-list(1L);attributes(x)<-list(class='box',names='a');identical(names(attributes(unclass(x))),'names')",
        "x<-list(1L);attributes(x)<-list(names='a',class='box');identical(names(attributes(unclass(x))),'names')",
        "x<-structure(list(1L,2L),names=c('',''),class='box');identical(names(unclass(x)),c('',''))",
        "x<-structure(list(),names=character(),class='box');identical(names(unclass(x)),character())",
    ] { truth(&mut f, source)?; }
    Ok(())
}

#[test]
fn unclass_closure_keeps_nonclass_metadata_environment_and_alias() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "f<-function(x=1L)x;attributes(f)<-list(before=1L,class='fun',after=list(a=2L));g<-f;u<-unclass(f);identical(names(attributes(u)),c('before','after')) && identical(f,g) && u()==1L && identical(environment(u),environment(f))")
}

#[test]
fn unclass_is_not_a_generic_and_copy_keeps_original_class() -> Result {
    let mut f = Frontend::default();
    truth(&mut f, "unclass.box<-function(x)stop('wrong dispatch');x<-structure(list(a=1L),class='box');u<-unclass(x);u$a<-2L;is.object(x) && x$a==1L && u$a==2L && !is.object(u)")
}

#[test]
fn unclass_returns_unclassed_values_and_plain_environment_unchanged() -> Result {
    let mut f = Frontend::default();
    for value in [
        "NULL",
        "TRUE",
        "1L",
        "1.5",
        "'s'",
        "quote(x)",
        "quote(f(x))",
        "expression(x)",
        "globalenv()",
        "matrix(1:4,2L)",
    ] {
        truth(&mut f, &format!("identical(unclass({value}),{value})"))?;
    }
    Ok(())
}

#[test]
fn unclass_checks_arity_exact_name_and_evaluates_before_name_check() -> Result {
    let mut f = Frontend::default();
    for (source, message) in [
        (
            "unclass()",
            "0 arguments passed to 'unclass' which requires 1",
        ),
        (
            "unclass(1L,2L)",
            "2 arguments passed to 'unclass' which requires 1",
        ),
        (
            "unclass(y=1L)",
            "supplied argument name 'y' does not match 'x'",
        ),
        (
            "unclass(xx=1L)",
            "supplied argument name 'xx' does not match 'x'",
        ),
        ("unclass(,)", "argument 1 is empty"),
    ] {
        assert!(f
            .eval_source(source)
            .expect_err("must reject")
            .to_string()
            .contains(message));
    }
    f.eval_source("i<-0L")?;
    assert!(f.eval_source("unclass(y={i<-1L;1L})").is_err());
    truth(&mut f, "i==1L")
}
