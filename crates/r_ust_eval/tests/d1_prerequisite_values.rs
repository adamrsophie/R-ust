#[path = "d1_prerequisites/class_assignment.rs"]
mod class_assignment;
#[path = "d1_prerequisites/lifetime.rs"]
mod lifetime;
#[path = "d1_prerequisites/syntax.rs"]
mod syntax;
#[path = "d1_prerequisites/typed.rs"]
mod typed;

use r_ust_eval::{EvalError, Frontend, OutputEvent};
use r_ust_sexp::{ListElement, Sexp};

fn text(values: &[&str]) -> Sexp {
    Sexp::String(values.iter().map(|value| Some((*value).into())).collect())
}

#[test]
fn classed_list_and_pairlist() {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("x<-structure(list(message='FAIL',call=NULL),class=c('simpleError','error','condition'));c(typeof(x),class(x),names(x));x$message;x[[1L]];x[['message',exact=TRUE]];is.object(x);is.list(x);mode(x);length(x)").unwrap();
    assert_eq!(
        values[1],
        text(&[
            "list",
            "simpleError",
            "error",
            "condition",
            "message",
            "call"
        ])
    );
    for value in &values[2..5] {
        assert_eq!(*value, text(&["FAIL"]));
    }
    assert_eq!(
        &values[5..],
        &[
            Sexp::Logical(vec![Some(true)]),
            Sexp::Logical(vec![Some(true)]),
            text(&["list"]),
            Sexp::Integer(vec![Some(2)])
        ]
    );
    let expected = Sexp::pairlist(vec![
        ListElement {
            name: Some("a".into()),
            value: Sexp::Integer(vec![Some(1)]),
        },
        ListElement {
            name: Some("b".into()),
            value: Sexp::Nil,
        },
    ]);
    assert_eq!(
        frontend
            .eval_source("pairlist(a=1L,b=NULL);pairlist();as.pairlist(list(a=1L,b=NULL))")
            .unwrap(),
        [expected.clone(), Sexp::Nil, expected]
    );
    assert!(frontend.runtime_mut().take_output().is_empty());
}

#[test]
fn class_removal_names_and_duplicate_attributes() {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("x<-structure(list(a=1L,b=2L),class='first',class='last');class(x);x$a<-3L;class(x);names(x)<-c('c','d');x[['c',exact=TRUE]];class(x)<-NULL;class(x);is.object(x);attributes(x)<-NULL;names(x);attributes(x)").unwrap();
    assert_eq!(values[1], text(&["last"]));
    assert_eq!(values[3], text(&["last"]));
    assert_eq!(values[5], Sexp::Integer(vec![Some(3)]));
    assert_eq!(values[7], text(&["list"]));
    assert_eq!(values[8], Sexp::Logical(vec![Some(false)]));
    assert_eq!(&values[10..], &[Sexp::Nil, Sexp::Nil]);
    assert_eq!(
        frontend
            .eval_source("names(structure(list(1L,2L),.Names=c('x','y')))")
            .unwrap(),
        [text(&["x", "y"])]
    );
}

#[test]
fn invalid_attributes_and_root_release() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("x<-structure(list(a=1L),class='old');saved<-x")
        .unwrap();
    for (source, message) in [
        (
            "attributes(x)<-{cat('RHS');list(1L,class='box')}",
            "attempt to use zero-length variable name",
        ),
        (
            "attributes(x)<-{cat('RHS');1L}",
            "attributes must be a list or NULL",
        ),
        (
            "attributes(x)<-{cat('RHS');list('new')}",
            "attributes must be named",
        ),
        (
            "attributes(x)<-{cat('RHS');list(class=1L)}",
            "attempt to set invalid 'class' attribute",
        ),
        (
            "attributes(x)<-{cat('RHS');list(class='new',2L)}",
            "all attributes must have names [2 does not]",
        ),
    ] {
        let error = frontend.eval_source(source).unwrap_err();
        assert!(error.to_string().ends_with(message), "{error}");
        assert_eq!(
            frontend.runtime_mut().take_output(),
            [OutputEvent::Stdout("RHS".into())]
        );
        assert_eq!(
            frontend
                .eval_source("identical(x,saved);sys.nframe()")
                .unwrap(),
            [
                Sexp::Logical(vec![Some(true)]),
                Sexp::Integer(vec![Some(0)])
            ]
        );
    }
    assert_eq!(
        frontend.eval_source("structure(NULL,class='x')"),
        Err(EvalError::Stop {
            message: "attempt to set an attribute on NULL".into(),
            call_requested: true
        })
    );
}

#[test]
fn ordinary_list_operations_preserve_their_existing_behavior() {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("x<-structure(list(a=1L,b=NA_integer_),class='box');length(x);unlist(x);as.integer(x);anyNA(x);is.na(x);lengths(x);x[1L];rep(x,2L);as.list(x);identical(x,structure(list(a=1L,b=NA_integer_),class='box'))").unwrap();
    assert_eq!(values[1], Sexp::Integer(vec![Some(2)]));
    assert_eq!(
        &values[2..4],
        &[
            Sexp::Integer(vec![Some(1), None]),
            Sexp::Integer(vec![Some(1), None])
        ]
    );
    assert_eq!(values[4], Sexp::Logical(vec![Some(true)]));
    assert_eq!(values[5], Sexp::Logical(vec![Some(false), Some(true)]));
    assert_eq!(values[6], Sexp::Integer(vec![Some(1), Some(1)]));
    assert!(matches!(&values[7], Sexp::List(elements) if elements.len() == 1));
    assert!(matches!(&values[8], Sexp::List(elements) if elements.len() == 4));
    assert!(matches!(&values[9], Sexp::ClassedList { elements, .. } if elements.len() == 2));
    assert_eq!(values[10], Sexp::Logical(vec![Some(true)]));
}

#[test]
fn attribute_insertion_order_survives_replacement_and_readdition() {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("x<-structure(list(1L),class='box',names='a');names(attributes(x));class(x)<-'new';names(attributes(x));names(x)<-NULL;names(x)<-'b';names(attributes(x));class(x)<-NULL;class(x)<-'last';names(attributes(x))").unwrap();
    for index in [1, 3, 6] {
        assert_eq!(values[index], text(&["class", "names"]));
    }
    assert_eq!(values[9], text(&["names", "class"]));
}

#[test]
fn unrepresented_attribute_paths_fail_explicitly_without_rebinding() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("x<-structure(list(a=1L),class='box');saved<-x")
        .unwrap();
    for source in [
        "class(x)<-1L",
        "names(x)<-NA_character_",
        "names(x)<-character(0)",
        "attributes(x)<-list(dim=1L)",
        "storage.mode(x)<-'integer'",
    ] {
        assert!(
            matches!(
                frontend.eval_source(source),
                Err(EvalError::NotImplemented(_))
            ),
            "{source}"
        );
        assert_eq!(
            frontend.eval_source("identical(x,saved)").unwrap(),
            [Sexp::Logical(vec![Some(true)])]
        );
    }
}

#[test]
fn classed_list_matrix_empty_storage_and_default_display() {
    let mut frontend = Frontend::default();
    let values = frontend
        .eval_source("x<-structure(list(a=1L),class='box');typeof(matrix(x,0L,0L));is.list(x)")
        .unwrap();
    assert_eq!(
        &values[1..],
        &[text(&["list"]), Sexp::Logical(vec![Some(true)])]
    );
    assert_eq!(
        r_ust_eval::format_sexp(&values[0]).unwrap(),
        "$a\n[1] 1\n\nattr(,\"class\")\n[1] \"box\""
    );
}
