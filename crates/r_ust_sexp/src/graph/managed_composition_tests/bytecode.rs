use super::*;

#[test]
fn managed_composition_bytecode_alias_keeps_pending_identity_and_source() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let code = code(&owner, integer(7))?;
    let raw = Sexp::Bytecode(code.clone());
    let before = live_nodes(&heap);
    let value = ValueObject::import(&owner, &raw, ValueImport::Explicit)?;
    assert_eq!(live_nodes(&heap), before);
    assert_eq!(value.sexptype(), SexpType::Bytecode);
    assert_eq!(value.transport()?, raw);
    let wrapped = Sexp::Managed(value.clone());
    let alias = ValueObject::import(&owner, &wrapped, ValueImport::Explicit)?;
    assert!(value.same_object(&alias));
    assert_eq!(live_nodes(&heap), before);
    assert_eq!(crate::bytecode_source(&wrapped)?, integer(7));
    assert_eq!(wrapped.snapshot()?, raw);
    owner.insert("code".into(), &raw)?;
    let count = value.semantic_reference_count()?;
    assert_eq!(owner.begin_assignment("code")?, Some(false));
    owner.insert("code".into(), &wrapped)?;
    assert_eq!(owner.begin_assignment("code")?, Some(true));
    assert_eq!(value.semantic_reference_count()?, count);
    assert!(owner.finish_assignment("code")?);
    let stored = owner.get_local("code")?.ok_or(GraphError::WrongKind)?;
    assert_eq!(stored.snapshot()?, raw);
    Ok(())
}

#[test]
fn managed_composition_aggregate_keeps_bytecode_edges_through_unclass_and_final_release() -> Result
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let code = code(&owner, integer(7))?;
    let child = ValueObject::import(&owner, &Sexp::Bytecode(code.clone()), ValueImport::Explicit)?;
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![Sexp::Bytecode(code.clone())]),
        attributes: vec![
            attribute("class", Sexp::String(vec![Some("custom".into())])),
            attribute("saved", Sexp::Managed(child.clone())),
        ],
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let retired = [value.0.id(), value.view()?.children[1].0.id()];
    assert_eq!(child.semantic_reference_count()?, 2);
    let unclassed = value.without_class()?;
    assert_eq!(child.semantic_reference_count()?, 4);
    assert!(value.view()?.children[0].same_object(&unclassed.view()?.children[0]));
    drop(raw);
    drop(value);
    assert_eq!(child.semantic_reference_count()?, 4);
    assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 2);
    for id in retired {
        assert_eq!(heap.lock().node(id).err(), Some(GraphError::StaleNode));
    }
    drop(code);
    drop(child);
    drop(owner);
    let snapshot = unclassed.snapshot()?;
    let Sexp::AttributedAggregate {
        values: AggregateVector::Vector(values),
        attributes,
    } = &snapshot
    else {
        panic!("aggregate")
    };
    let Sexp::Bytecode(payload) = &values[0] else {
        panic!("payload bytecode")
    };
    let Sexp::Bytecode(attribute) = &attributes[0].value else {
        panic!("attribute bytecode")
    };
    assert!(payload.same_object(attribute));
    assert_eq!(payload.source()?, integer(7));
    drop(snapshot);
    drop(unclassed);
    assert_eq!(live_nodes(&heap), 0);
    Ok(())
}

#[test]
fn managed_composition_foreign_bytecode_rejects_entire_typed_batch() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let foreign_heap = Arc::new(Heap::default());
    let foreign = foreign_heap.session();
    let foreign_code = code(&foreign, integer(7))?;
    let wrapped = Sexp::Managed(ValueObject::import(
        &foreign,
        &Sexp::Bytecode(foreign_code),
        ValueImport::Explicit,
    )?);
    let values = [
        (integer(1), ValueImport::Evaluated),
        (
            Sexp::AttributedAggregate {
                values: AggregateVector::Expression(vec![integer(2)]),
                attributes: vec![attribute("saved", wrapped)],
            },
            ValueImport::Code,
        ),
    ];
    let before = live_nodes(&heap);
    assert_eq!(
        ValueObject::import_many(&owner, &values),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(live_nodes(&heap), before);
    Ok(())
}
