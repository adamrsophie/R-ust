use r_ust_eval::{format_sexp, Frontend, RuntimeState};
use r_ust_sexp::{
    AssignmentOperator, CallArgument, Environment, ListElement, ListObject, Sexp, SexpType,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn slot(name: Option<&str>, value: Sexp) -> ListElement {
    ListElement {
        name: name.map(str::to_owned),
        value,
    }
}
fn bind(runtime: &mut RuntimeState, value: Sexp) -> Result {
    runtime.eval(&Sexp::Assignment {
        operator: AssignmentOperator::Left,
        target: "x".into(),
        value: Box::new(value),
    })?;
    Ok(())
}
fn truth(frontend: &mut Frontend, source: &str) -> Result {
    assert_eq!(
        frontend.eval_source(source)?.last(),
        Some(&Sexp::Logical(vec![Some(true)])),
        "{source}"
    );
    Ok(())
}

#[test]
fn legacy_constructors_and_explicit_variant_share_language_identity() -> Result {
    let elements = vec![
        slot(None, Sexp::Integer(vec![Some(1)])),
        slot(Some(""), Sexp::Integer(vec![Some(2)])),
    ];
    let legacy = Sexp::List(elements.clone());
    let explicit = Sexp::NamedList {
        elements,
        class: None,
        class_first: false,
    };
    let mut runtime = RuntimeState::default();
    assert_eq!(
        runtime.eval(&Sexp::Call {
            function: Box::new(Sexp::Symbol("identical".into())),
            arguments: vec![
                CallArgument::positional(legacy),
                CallArgument::positional(explicit)
            ]
        })?,
        Sexp::Logical(vec![Some(true)])
    );
    for value in [
        Sexp::List(vec![]),
        Sexp::list_with_metadata(vec![], true, None),
    ] {
        assert_eq!(value.sexptype(), SexpType::Vector);
        let mut frontend = Frontend::default();
        bind(frontend.runtime_mut(), value)?;
        truth(
            &mut frontend,
            "is.list(x) && is.vector(x) && typeof(x)=='list' && mode(x)=='list' && length(x)==0L",
        )?;
    }
    Ok(())
}

#[test]
fn classed_legacy_and_named_class_views_preserve_order_and_identity() -> Result {
    let class = Sexp::String(vec![Some("box".into())]);
    let empty = Sexp::list_with_metadata(vec![], true, Some((class, true)));
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), empty)?;
    truth(&mut frontend,"is.object(x) && !is.vector(x) && identical(class(x),'box') && identical(names(x),character()) && identical(names(attributes(x)),c('class','names'))")?;
    truth(
        &mut frontend,
        "class(x)<-NULL;!is.object(x) && is.vector(x) && identical(names(x),character())",
    )?;
    Ok(())
}

#[test]
fn graph_projection_and_copy_on_names_assignment_preserve_original() -> Result {
    let owner = Environment::global();
    let value =
        Sexp::list_with_metadata(vec![slot(None, Sexp::Integer(vec![Some(7)]))], true, None);
    let object = ListObject::from_value(&owner, &value)?;
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), Sexp::ListObject(object.clone()))?;
    truth(&mut frontend,"y<-x;names(y)<-'changed';identical(names(x),NA_character_) && identical(names(y),'changed')")?;
    assert_eq!(object.value()?, value);
    assert_eq!(format_sexp(&Sexp::ListObject(object))?, "$<NA>\n[1] 7\n");
    Ok(())
}

#[test]
fn list_to_data_mask_keeps_na_name_literal_binding() -> Result {
    let mut frontend = Frontend::default();
    bind(
        frontend.runtime_mut(),
        Sexp::list_with_metadata(vec![slot(None, Sexp::Integer(vec![Some(9)]))], true, None),
    )?;
    truth(&mut frontend, "with(x,`NA`)==9L")?;
    Ok(())
}

#[test]
fn unnamed_subset_preserves_missing_numeric_and_logical_slots() -> Result {
    let mut frontend = Frontend::default();
    truth(
        &mut frontend,
        "x<-list(7L);identical(x[c(1L,3L,NA_integer_,0L)],list(7L,NULL,NULL))",
    )?;
    truth(
        &mut frontend,
        "identical(x[c(TRUE,NA,TRUE,FALSE)],list(7L,NULL,NULL))",
    )?;
    truth(
        &mut frontend,
        "is.null(names(x[c(1L,3L,NA_integer_)])) && identical(x[c(-2L,0L)],x)",
    )?;
    Ok(())
}

#[test]
fn named_subset_preserves_missing_slots_and_na_names() -> Result {
    let mut frontend = Frontend::default();
    truth(&mut frontend, "x<-list(a=7L);y<-x[c(1L,3L,NA_integer_)];length(y)==3L && identical(names(y),c('a',NA_character_,NA_character_)) && is.null(y[[2L]]) && is.null(y[[3L]])")?;
    truth(&mut frontend, "y<-x[c(TRUE,NA,TRUE)];length(y)==3L && identical(names(y),c('a',NA_character_,NA_character_)) && is.null(y[[2L]]) && is.null(y[[3L]])")?;
    Ok(())
}
