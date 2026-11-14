use r_ust_eval::{EvalError, Frontend};
use r_ust_sexp::Sexp;

#[test]
fn list_class_assignment_removes_attribute_but_attributes_setter_does_not() {
    let mut frontend = Frontend::default();
    let values = frontend.eval_source("x<-structure(list(a=1L),class='box');class(x)<-'list';class(x);names(attributes(x));is.object(x);attributes(x)<-list(class='list');is.object(x)").unwrap();
    assert_eq!(values[2], Sexp::String(vec![Some("list".into())]));
    assert_eq!(values[3], Sexp::String(vec![Some("names".into())]));
    assert_eq!(values[4], Sexp::Logical(vec![Some(false)]));
    assert_eq!(values[6], Sexp::Logical(vec![Some(true)]));
    assert!(matches!(&values[0], Sexp::ClassedList { .. }));
}

#[test]
fn reserved_class_validation_is_atomic_and_source_correct() {
    let mut frontend = Frontend::default();
    frontend
        .eval_source("x<-structure(list(a=1L),class='box');saved<-x")
        .unwrap();
    for (class, message) in [
        ("matrix", "invalid to set the class to matrix unless the dimension attribute is of length 2 (was 0)"),
        ("array", "cannot set class to \"array\" unless the dimension attribute has length > 0"),
        ("environment", "\"environment\" can only be set as the class if the object has this type; found \"list\""),
    ] {
        assert_eq!(frontend.eval_source(&format!("class(x)<-'{class}'")), Err(EvalError::Domain(message.into())));
        assert_eq!(frontend.eval_source("identical(x,saved)").unwrap(), [Sexp::Logical(vec![Some(true)])]);
    }
    for class in [
        "logical",
        "integer",
        "double",
        "raw",
        "complex",
        "character",
        "expression",
        "char",
        "numeric",
    ] {
        assert!(matches!(
            frontend.eval_source(&format!("class(x)<-'{class}'")),
            Err(EvalError::NotImplemented(
                "class replacement type conversion"
            ))
        ));
        assert_eq!(
            frontend.eval_source("identical(x,saved)").unwrap(),
            [Sexp::Logical(vec![Some(true)])]
        );
    }
}

#[test]
fn multiple_class_strings_follow_the_s3_path_except_leading_matrix() {
    let mut frontend = Frontend::default();
    assert_eq!(
        frontend
            .eval_source("x<-list(1L);class(x)<-c('list','box');is.object(x);class(x)")
            .unwrap()[2..],
        [
            Sexp::Logical(vec![Some(true)]),
            Sexp::String(vec![Some("list".into()), Some("box".into())])
        ]
    );
    assert!(frontend.eval_source("class(x)<-c('matrix','box')").is_err());
    assert!(frontend.eval_source("class(x)<-c('array','box')").is_ok());
}
