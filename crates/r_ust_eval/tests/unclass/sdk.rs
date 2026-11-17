use super::*;
use r_ust_sexp::{Environment, ListObject};

fn owner(frontend: &mut Frontend) -> Result<Environment> {
    let closure = frontend
        .eval_source("function()NULL")?
        .pop()
        .ok_or("closure missing")?;
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(owner),
        ..
    } = closure
    else {
        panic!("captured owner")
    };
    Ok(owner)
}

#[test]
fn unclass_graph_list_copy_preserves_child_identity_and_alias_metadata() -> Result {
    let mut f = Frontend::default();
    let owner = owner(&mut f)?;
    let child = owner.child()?;
    child.insert("x".into(), &Sexp::Integer(vec![Some(7)]))?;
    let raw = Sexp::list_with_metadata(
        vec![attribute("payload", Sexp::Environment(child.clone()))],
        true,
        Some((strings(&["box"]), false)),
    );
    let object = ListObject::from_value(&owner, &raw)?;
    bind(f.runtime_mut(), "x", Sexp::ListObject(object.clone()))?;
    f.eval_source("u<-unclass(x)")?;
    let Sexp::ListObject(result) = f.runtime().get("u")?.ok_or("result missing")? else {
        panic!("graph list")
    };
    assert!(!result.same_object(&object));
    assert_eq!(object.value()?, raw);
    assert_eq!(result.element(0)?, Sexp::Environment(child.clone()));
    assert!(result
        .value()?
        .ordinary_list()
        .ok_or("list missing")?
        .class
        .is_none());
    drop(f);
    drop(owner);
    drop(object);
    assert_eq!(result.element(0)?, Sexp::Environment(child.clone()));
    assert_eq!(child.get_local("x")?, Some(Sexp::Integer(vec![Some(7)])));
    Ok(())
}

#[test]
fn unclass_graph_list_without_class_returns_original_handle() -> Result {
    let mut f = Frontend::default();
    let owner = owner(&mut f)?;
    let object = ListObject::from_value(&owner, &Sexp::List(vec![]))?;
    bind(f.runtime_mut(), "x", Sexp::ListObject(object.clone()))?;
    let result = f.eval_source("unclass(x)")?.pop().ok_or("result missing")?;
    let Sexp::ListObject(result) = result else {
        panic!("graph list")
    };
    assert!(result.same_object(&object));
    Ok(())
}

#[test]
fn unclass_closure_custom_graph_attribute_survives_origin_runtime() -> Result {
    let result;
    let child;
    {
        let mut f = Frontend::default();
        let owner = owner(&mut f)?;
        child = owner.child()?;
        child.insert("x".into(), &Sexp::Integer(vec![Some(42)]))?;
        let closure = Sexp::Closure {
            parameters: vec![],
            body: Box::new(Sexp::Integer(vec![Some(1)])),
            environment: ClosureEnvironment::Captured(owner),
            attributes: vec![
                attribute("before", Sexp::Environment(child.clone())),
                attribute("class", strings(&["box"])),
                attribute("after", Sexp::Integer(vec![Some(2)])),
            ],
        };
        bind(f.runtime_mut(), "x", closure.clone())?;
        result = f.eval_source("unclass(x)")?.pop().ok_or("result missing")?;
        assert_eq!(f.runtime().get("x")?, Some(closure));
    }
    let Sexp::Closure { attributes, .. } = result else {
        panic!("closure")
    };
    assert_eq!(
        attributes
            .iter()
            .map(|a| a.name.as_deref())
            .collect::<Vec<_>>(),
        [Some("before"), Some("after")]
    );
    assert_eq!(attributes[0].value, Sexp::Environment(child.clone()));
    assert_eq!(child.get_local("x")?, Some(Sexp::Integer(vec![Some(42)])));
    Ok(())
}
