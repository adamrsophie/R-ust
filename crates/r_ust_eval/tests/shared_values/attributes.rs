use super::*;

fn classed(value: Sexp) -> Sexp {
    Sexp::ClassedList {
        elements: vec![],
        class: Box::new(Sexp::AttributedCharacter {
            values: vec![Some("box".into())],
            attributes: vec![ListElement {
                name: Some("custom".into()),
                value,
            }],
        }),
        class_first: true,
    }
}

#[test]
fn boxed_class_attributes_preserve_graph_identity_and_detect_cycles() -> Result {
    let owner = Environment::global();
    let nested = ListObject::from_value(&owner, &payload())?;
    let graph = ListObject::from_value(&owner, &classed(Sexp::ListObject(nested.clone())))?;
    graph.set_gp_shared(5)?;
    let mut frontend = Frontend::default();
    bind(frontend.runtime_mut(), Sexp::ListObject(graph.clone()))?;
    let result = frontend.eval_source("attr(class(x),'custom')")?;
    let [Sexp::ListObject(retained)] = result.as_slice() else {
        panic!("nested graph class attribute")
    };
    assert!(retained.same_object(&nested));
    frontend.eval_source("x$a<-1L")?;
    let Some(Sexp::ListObject(copy)) = frontend.runtime().get("x")? else {
        panic!("copy")
    };
    assert!(!copy.same_object(&graph));
    assert_eq!(copy.header()?.gp(), 5);
    assert_eq!(graph.value()?, classed(Sexp::ListObject(nested.clone())));
    let left = ListObject::from_value(&owner, &payload())?;
    let right = ListObject::from_value(&owner, &payload())?;
    left.replace_shared(&[(0, classed(Sexp::ListObject(left.clone())))])?;
    right.replace_shared(&[(0, classed(Sexp::ListObject(right.clone())))])?;
    assert_eq!(
        call(
            frontend.runtime_mut(),
            "identical",
            vec![Sexp::ListObject(left.clone()), Sexp::ListObject(right)]
        )?,
        Sexp::Logical(vec![Some(true)])
    );
    bind(frontend.runtime_mut(), Sexp::ListObject(left.clone()))?;
    let error = frontend.eval_source("print(x)").expect_err("cyclic print");
    assert!(error.to_string().contains("cyclic ordinary value printing"));
    assert!(frontend.runtime_mut().take_output().is_empty());
    assert_eq!(
        format_sexp(&Sexp::ListObject(left)),
        Err(EvalError::NotImplemented("cyclic ordinary value printing"))
    );
    Ok(())
}
