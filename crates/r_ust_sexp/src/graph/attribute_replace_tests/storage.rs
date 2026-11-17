use super::*;

#[test]
fn attribute_replace_persistent_copy_keeps_children_order_headers_and_semantic_owners() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let child = ValueObject::import(&owner, &integer(7), ValueImport::Evaluated)?;
    let raw = vector(
        vec![child.transport()?],
        vec![
            attribute("note", child.transport()?),
            attribute("class", text("custom")),
        ],
    );
    let original = ValueObject::import(&owner, &raw, ValueImport::Code)?;
    set_gp(&original, 0x2040)?;
    let expected_snapshot = original.snapshot()?;
    let alias = original.clone();
    let before = original.view()?;
    let mut attrs = original.attribute_values()?;
    attrs.reverse();
    let count = live(&heap);
    let copy = original.with_attributes(&attrs)?;
    let after = copy.view()?;
    assert_eq!(live(&heap), count + 2);
    assert!(!original.same_object(&copy));
    assert!(original.same_object(&alias));
    assert_eq!(after.header, before.header);
    assert_eq!(after.kind, before.kind);
    assert_eq!(after.mode, before.mode);
    assert!(before.children[0].same_object(&after.children[0]));
    assert!(!before.children[1].same_object(&after.children[1]));
    let old_attributes = before.children[1].view()?;
    let new_attributes = after.children[1].view()?;
    assert_eq!(new_attributes.kind, ValueKind::Attributes);
    assert!(old_attributes.children[0].same_object(&new_attributes.children[1]));
    assert!(old_attributes.children[1].same_object(&new_attributes.children[0]));
    assert_eq!(child.semantic_reference_count()?, 4);
    assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 4);
    assert_eq!(
        snapshots(&copy.attribute_values()?)?,
        vec![
            attribute("class", text("custom")),
            attribute("note", integer(7))
        ]
    );
    assert_eq!(original.snapshot()?, expected_snapshot);
    drop(old_attributes);
    drop(new_attributes);
    drop(before);
    drop(after);
    drop(attrs);
    drop(original);
    drop(alias);
    assert_eq!(child.semantic_reference_count()?, 4);
    assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 2);
    let cleared = copy.with_attributes(&[])?;
    assert!(!cleared.view()?.header.is_object());
    assert_eq!(cleared.view()?.header.gp(), 0x2040);
    assert!(cleared.attribute_values()?.is_empty());
    assert_eq!(child.semantic_reference_count()?, 5);
    assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 3);
    drop(copy);
    assert_eq!(child.semantic_reference_count()?, 5);
    assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 1);
    drop(cleared);
    assert_eq!(child.semantic_reference_count()?, 5);
    assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 0);
    Ok(())
}

#[test]
fn attribute_replace_closure_retains_formals_body_environment_and_metadata() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let raw = Sexp::Closure {
        parameters: vec![FunctionParameter::optional("x", integer(1))],
        body: Box::new(Sexp::Symbol("x".into())),
        environment: ClosureEnvironment::Captured(owner.clone()),
        attributes: vec![attribute("note", text("old"))],
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Code)?;
    set_gp(&value, 0x2040)?;
    let before = value.view()?;
    let copy = value.with_attributes(&[
        attribute("class", text("custom")),
        attribute("note", text("new")),
    ])?;
    let after = copy.view()?;
    for index in 0..3 {
        assert!(before.children[index].same_object(&after.children[index]));
    }
    assert!(!before.children[3].same_object(&after.children[3]));
    assert_eq!(after.header.sexptype(), SexpType::Closure);
    assert_eq!(after.header.gp(), 0x2040);
    assert!(after.header.is_object());
    assert_eq!(after.kind, before.kind);
    assert_eq!(after.mode, before.mode);
    assert_eq!(value.snapshot()?, raw);
    assert_eq!(
        snapshots(&copy.attribute_values()?)?,
        vec![
            attribute("class", text("custom")),
            attribute("note", text("new"))
        ]
    );
    Ok(())
}

#[test]
fn attribute_replace_typed_vectors_preserve_payload_and_explicit_import_modes() -> Result {
    let owner = Environment::global();
    let inputs = [
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(3)]),
            attributes: vec![],
        },
        Sexp::AttributedCharacter {
            values: vec![Some("a".into())],
            attributes: vec![],
        },
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(vec![Sexp::Symbol("x".into())]),
            attributes: vec![],
        },
    ];
    for input in inputs {
        let value = ValueObject::import(&owner, &input, ValueImport::Evaluated)?;
        let before = value.view()?;
        let alias = ValueObject::import(&owner, &value.transport()?, ValueImport::Explicit)?;
        assert!(value.same_object(&alias));
        let copy = alias.with_attributes(&[
            attribute("names", text("named")),
            attribute("class", text("custom")),
        ])?;
        let after = copy.view()?;
        assert_eq!(after.header.sexptype(), before.header.sexptype());
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.mode, before.mode);
        for index in 0..before.children.len() - 1 {
            assert!(before.children[index].same_object(&after.children[index]));
        }
        assert!(after.header.is_object());
        assert_eq!(value.snapshot()?, input);
        assert_eq!(
            snapshots(&copy.attribute_values()?)?,
            vec![
                attribute("names", text("named")),
                attribute("class", text("custom"))
            ]
        );
    }
    Ok(())
}
