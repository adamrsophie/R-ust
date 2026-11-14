use r_ust_eval::{Frontend, RuntimeState};
use r_ust_sexp::{AssignmentOperator, CallArgument, Sexp};

#[test]
fn classed_nested_environment_and_closure_outlive_origin() {
    let value = {
        let mut frontend = Frontend::default();
        frontend
            .eval_source("x<-17L;e<-environment();structure(list(e=e,f=function()x),class='box')")
            .unwrap()
            .pop()
            .unwrap()
    };
    let alias = value.clone();
    drop(value);
    let mut other = Frontend::default();
    other.eval_source("x<-99L").unwrap();
    other
        .runtime_mut()
        .eval(&Sexp::Assignment {
            operator: AssignmentOperator::Left,
            target: "retained".into(),
            value: Box::new(alias),
        })
        .unwrap();
    assert_eq!(
        other
            .eval_source("retained$e$x;retained$f();class(retained);sys.nframe()")
            .unwrap(),
        [
            Sexp::Integer(vec![Some(17)]),
            Sexp::Integer(vec![Some(17)]),
            Sexp::String(vec![Some("box".into())]),
            Sexp::Integer(vec![Some(0)])
        ]
    );
}

#[test]
fn sdk_pairlist_conversion_keeps_missing_values_and_tags() {
    let elements = vec![
        r_ust_sexp::ListElement {
            name: Some("missing".into()),
            value: Sexp::MissingValue,
        },
        r_ust_sexp::ListElement {
            name: Some("nil".into()),
            value: Sexp::Nil,
        },
    ];
    let source = Sexp::List(elements.clone());
    let expected = Sexp::pairlist(elements);
    let mut runtime = RuntimeState::default();
    let result = runtime
        .eval(&Sexp::InternalCall {
            operation: r_ust_sexp::InternalOperation::AsVector,
            arguments: vec![
                CallArgument::positional(source),
                CallArgument::positional(Sexp::String(vec![Some("pairlist".into())])),
            ],
        })
        .unwrap();
    assert_eq!(result, expected);
}
