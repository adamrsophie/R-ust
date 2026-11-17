use super::*;

#[test]
fn attribute_replace_payload_metadata_cycle_survives_origin_drop_then_collects() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let child = owner.child()?;
    child.insert("marker".into(), &integer(11))?;
    let original = ValueObject::import(
        &owner,
        &vector(
            vec![Sexp::Environment(child.clone())],
            vec![attribute("class", text("old"))],
        ),
        ValueImport::Evaluated,
    )?;
    let replacement =
        original.with_attributes(&[attribute("note", Sexp::Environment(child.clone()))])?;
    child.insert("cycle".into(), &replacement.transport()?)?;
    let original_id = original.0.id();
    let ids = [owner.0.id(), child.0.id(), replacement.0.id()];
    drop(original);
    drop(child);
    drop(owner);
    assert!(matches!(
        heap.lock().node(original_id),
        Err(GraphError::StaleNode)
    ));
    let borrowed = replacement.borrow()?;
    let Sexp::AttributedAggregate {
        values: AggregateVector::Vector(values),
        attributes,
    } = borrowed.value()
    else {
        panic!("aggregate")
    };
    let Sexp::Environment(child) = &values[0] else {
        panic!("payload environment")
    };
    assert_eq!(child.get_local("marker")?, Some(integer(11)));
    assert_eq!(attributes[0].value, Sexp::Environment(child.clone()));
    let Sexp::Managed(cycle) = child.get_local("cycle")?.ok_or(GraphError::WrongKind)? else {
        panic!("cycle")
    };
    assert!(cycle.same_object(&replacement));
    drop(cycle);
    drop(borrowed);
    drop(replacement);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    assert_eq!(live(&heap), 0);
    Ok(())
}

#[test]
fn attribute_replace_bytecode_payload_and_metadata_preserve_opaque_identity() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let code = crate::BytecodeObject::new(
        &owner,
        crate::BytecodeParts {
            code: vec![12, 1],
            constants: vec![integer(7)],
            expression: integer(7),
        },
    )?;
    let handle = ValueObject::import(&owner, &Sexp::Bytecode(code.clone()), ValueImport::Explicit)?;
    let original = ValueObject::import(
        &owner,
        &vector(vec![Sexp::Bytecode(code.clone())], vec![]),
        ValueImport::Code,
    )?;
    let replacement =
        original.with_attributes(&[attribute("saved", Sexp::Managed(handle.clone()))])?;
    assert_eq!(handle.semantic_reference_count()?, 3);
    assert_eq!(heap.lock().audit_reference_count(handle.0.id())?, 3);
    assert!(original.view()?.children[0].same_object(&replacement.view()?.children[0]));
    drop(original);
    assert_eq!(handle.semantic_reference_count()?, 3);
    assert_eq!(heap.lock().audit_reference_count(handle.0.id())?, 2);
    drop(code);
    drop(handle);
    drop(owner);
    let snapshot = replacement.snapshot()?;
    let Sexp::AttributedAggregate {
        values: AggregateVector::Vector(values),
        attributes,
    } = &snapshot
    else {
        panic!("aggregate")
    };
    let Sexp::Bytecode(payload) = &values[0] else {
        panic!("bytecode payload")
    };
    let Sexp::Bytecode(metadata) = &attributes[0].value else {
        panic!("bytecode metadata")
    };
    assert!(payload.same_object(metadata));
    assert_eq!(payload.source()?, integer(7));
    drop(snapshot);
    drop(replacement);
    assert_eq!(live(&heap), 0);
    Ok(())
}
