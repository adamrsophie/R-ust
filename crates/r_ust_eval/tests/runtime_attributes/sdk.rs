use super::*;

#[test]
fn attributed_atomic_payload_types_survive_arbitrary_metadata() -> Result {
    for (values, kind) in [
        (
            AtomicVector::Integer(vec![Some(1), None]),
            SexpType::Integer,
        ),
        (AtomicVector::Real(vec![1.5, 2.5]), SexpType::Real),
        (
            AtomicVector::Logical(vec![Some(true), None]),
            SexpType::Logical,
        ),
        (
            AtomicVector::String(vec![Some("a".into()), None]),
            SexpType::String,
        ),
    ] {
        let raw = Sexp::AttributedAtomic {
            values,
            attributes: vec![attr("note", string("original"))],
        };
        let mut f = Frontend::default();
        bind(f.runtime_mut(), "x", raw.clone())?;
        truth(&mut f,"y<-x;attr(y,'note')<-'changed';identical(attr(x,'note'),'original') && identical(attr(y,'note'),'changed') && length(x)==2L && length(y)==2L")?;
        let result = f.runtime().get("y")?.ok_or("missing y")?;
        assert_eq!(result.sexptype(), kind);
        let Some(Sexp::AttributedAtomic {
            values: changed, ..
        }) = result.attribute_description()?
        else {
            panic!("atomic description")
        };
        let Some(Sexp::AttributedAtomic {
            values: original, ..
        }) = raw.attribute_description()?
        else {
            panic!("original description")
        };
        assert_eq!(changed, original);
    }
    Ok(())
}

#[test]
fn expression_attributes_keep_expression_payload_and_inert_code() -> Result {
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Expression(vec![Sexp::Symbol("never_evaluate".into())]),
        attributes: vec![attr("note", string("original"))],
    };
    let mut f = Frontend::default();
    bind(f.runtime_mut(), "x", raw)?;
    truth(&mut f,"y<-x;names(y)<-'a';typeof(y)=='expression' && identical(attr(x,'note'),'original') && identical(names(y),'a') && is.null(names(x))")?;
    let result = f.runtime().get("y")?.ok_or("missing y")?;
    let Some(Sexp::AttributedAggregate {
        values: AggregateVector::Expression(values),
        ..
    }) = result.attribute_description()?
    else {
        panic!("expression payload")
    };
    assert_eq!(values, vec![Sexp::Symbol("never_evaluate".into())]);
    Ok(())
}

#[test]
fn managed_metadata_copy_keeps_payload_and_outlives_runtime() -> Result {
    let owner = Environment::global();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1), None]),
        attributes: vec![attr("note", string("original"))],
    };
    let handle = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let before = handle.view()?;
    let mut f = Frontend::default();
    bind(f.runtime_mut(), "x", Sexp::Managed(handle.clone()))?;
    truth(&mut f,"y<-x;attr(y,'note')<-'changed';identical(attr(x,'note'),'original') && identical(attr(y,'note'),'changed') && typeof(y)=='integer'")?;
    let Sexp::Managed(changed) = f.runtime().get("y")?.ok_or("missing y")? else {
        panic!("managed setter result")
    };
    assert!(!handle.same_object(&changed));
    assert_eq!(handle.snapshot()?, raw);
    assert_eq!(changed.view()?.mode, before.mode);
    drop(f);
    drop(owner);
    let Some(Sexp::AttributedAtomic { values, attributes }) =
        changed.snapshot()?.attribute_description()?
    else {
        panic!("retained metadata")
    };
    assert_eq!(values, AtomicVector::Integer(vec![Some(1), None]));
    assert_eq!(attributes, vec![attr("note", string("changed"))]);
    assert_eq!(handle.snapshot()?, raw);
    Ok(())
}

#[test]
fn managed_list_metadata_preserves_reference_child_identity() -> Result {
    let owner = Environment::global();
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::Environment(owner.clone())]),
        attributes: vec![attr("note", string("original"))],
    };
    let handle = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let mut f = Frontend::default();
    bind(f.runtime_mut(), "x", Sexp::Managed(handle.clone()))?;
    truth(
        &mut f,
        "y<-x;attr(y,'note')<-'changed';identical(attr(y,'note'),'changed')",
    )?;
    let Sexp::Managed(changed) = f.runtime().get("y")?.ok_or("missing y")? else {
        panic!("managed result")
    };
    let Some(Sexp::AttributedAggregate {
        values: AggregateVector::Vector(values),
        ..
    }) = changed.snapshot()?.attribute_description()?
    else {
        panic!("aggregate payload")
    };
    assert_eq!(values, vec![Sexp::Environment(owner)]);
    assert_eq!(handle.snapshot()?, raw);
    Ok(())
}
