use super::*;

#[test]
fn general_attributes_payload_and_attribute_children_have_distinct_persistent_owners() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let child = ValueObject::import(&owner, &integer(7), ValueImport::Evaluated)?;
    let custom = ValueObject::import(&owner, &text("kept"), ValueImport::Evaluated)?;
    let raw = vector(
        vec![child.transport()?],
        vec![
            attribute("class", text("custom")),
            attribute("note", custom.transport()?),
        ],
    );
    let value = ValueObject::import(&owner, &raw, ValueImport::Code)?;
    let before = value.view()?;
    assert_eq!(before.children.len(), 2);
    assert!(before.children[0].same_object(&child));
    assert_eq!(before.children[1].view()?.kind, ValueKind::Attributes);
    assert_eq!(child.semantic_reference_count()?, 1);
    assert_eq!(custom.semantic_reference_count()?, 1);
    let out = value.without_class()?;
    let after = out.view()?;
    assert_eq!(after.mode, Some(ValueImport::Code));
    assert!(before.children[0].same_object(&after.children[0]));
    assert!(!before.children[1].same_object(&after.children[1]));
    assert!(after.children[1].view()?.children[0].same_object(&custom));
    assert_eq!(child.semantic_reference_count()?, 2);
    assert_eq!(custom.semantic_reference_count()?, 2);
    let alias = out.clone();
    let allocations = heap
        .lock()
        .slots
        .iter()
        .filter(|slot| slot.node.is_some())
        .count();
    let noop = out.without_class()?;
    assert!(out.same_object(&alias));
    assert!(out.same_object(&noop));
    assert_eq!(
        allocations,
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count()
    );
    assert_eq!(child.semantic_reference_count()?, 2);
    let retired = [value.0.id(), before.children[1].0.id()];
    drop(before);
    drop(after);
    drop(value);
    // GNU GC preserves surviving reference history.
    assert_eq!(child.semantic_reference_count()?, 2);
    assert_eq!(custom.semantic_reference_count()?, 2);
    assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 1);
    assert_eq!(heap.lock().audit_reference_count(custom.0.id())?, 1);
    for id in retired {
        assert_eq!(heap.lock().node(id).err(), Some(GraphError::StaleNode));
    }
    drop(raw);
    drop(noop);
    drop(alias);
    drop(out);
    drop(child);
    drop(custom);
    drop(owner);
    assert_eq!(
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count(),
        0
    );
    Ok(())
}

#[test]
fn general_attributes_expression_modes_and_nonclass_metadata_survive() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Expression(vec![Sexp::Symbol("x".into()), integer(1)]),
        attributes: vec![
            attribute("names", Sexp::String(vec![None, Some("".into())])),
            attribute("class", text("custom")),
            attribute("note", text("kept")),
        ],
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let view = value.view()?;
    assert_eq!(view.header.sexptype(), SexpType::Expression);
    assert_eq!(view.children.len(), 3);
    for child in &view.children[..2] {
        assert_eq!(child.view()?.mode, Some(ValueImport::Code));
    }
    assert_eq!(view.children[2].view()?.mode, Some(ValueImport::Evaluated));
    let out = value.without_class()?;
    assert_eq!(out.view()?.header.sexptype(), SexpType::Expression);
    assert_eq!(
        snapshots(&out.attribute_values()?)?,
        vec![
            attribute("names", Sexp::String(vec![None, Some("".into())])),
            attribute("note", text("kept"))
        ]
    );
    assert_eq!(value.snapshot()?, raw);
    Ok(())
}

#[test]
fn general_attributes_legacy_managed_list_dataframe_and_character_remove_only_class() -> Result {
    let owner = Environment::global();
    let inputs = [
        Sexp::NamedList {
            elements: vec![ListElement {
                name: None,
                value: integer(1),
            }],
            class: Some(Box::new(text("custom"))),
            class_first: true,
        },
        Sexp::DataFrame {
            columns: vec![ListElement {
                name: Some("a".into()),
                value: integer(1),
            }],
            nrow: 1,
        },
        Sexp::AttributedCharacter {
            values: vec![Some("value".into())],
            attributes: vec![
                attribute("class", text("custom")),
                attribute("note", text("kept")),
            ],
        },
    ];
    for input in inputs {
        let value = ValueObject::import(&owner, &input, ValueImport::Evaluated)?;
        assert!(value.view()?.header.is_object());
        let mut expected = snapshots(&value.attribute_values()?)?;
        expected.retain(|a| a.name.as_deref() != Some("class"));
        let out = value.without_class()?;
        assert!(!value.same_object(&out));
        assert!(!out.view()?.header.is_object());
        assert_eq!(snapshots(&out.attribute_values()?)?, expected);
        assert_eq!(value.snapshot()?, input);
    }
    Ok(())
}

#[test]
fn general_attributes_closure_copy_preserves_formals_body_environment_and_gp() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let closure = Sexp::Closure {
        parameters: vec![crate::FunctionParameter::optional("x", integer(1))],
        body: Box::new(Sexp::Symbol("x".into())),
        environment: crate::ClosureEnvironment::Captured(owner.clone()),
        attributes: vec![
            attribute("class", text("custom")),
            attribute("note", text("kept")),
        ],
    };
    let value = ValueObject::import(&owner, &closure, ValueImport::Code)?;
    {
        let mut store = heap.lock();
        let Node::Value(node) = store.node_mut(value.0.id())? else {
            panic!("value")
        };
        node.header.set_gp(0x2040);
    }
    let before = value.view()?;
    let out = value.without_class()?;
    let after = out.view()?;
    assert_eq!(after.header.gp(), 0x2040);
    assert!(!after.header.is_object());
    assert_eq!(after.mode, Some(ValueImport::Code));
    for index in 0..3 {
        assert!(before.children[index].same_object(&after.children[index]));
    }
    assert!(!before.children[3].same_object(&after.children[3]));
    assert_eq!(
        snapshots(&out.attribute_values()?)?,
        vec![attribute("note", text("kept"))]
    );
    assert_eq!(value.snapshot()?, closure);
    Ok(())
}

#[test]
fn general_attributes_classless_environment_and_plain_value_keep_exact_identity() -> Result {
    let owner = Environment::global();
    for raw in [
        Sexp::Environment(owner.clone()),
        integer(1),
        vector(vec![], vec![]),
    ] {
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        let out = value.without_class()?;
        assert!(value.same_object(&out));
        assert_eq!(value.snapshot()?, out.snapshot()?);
        if let Sexp::Environment(environment) = out.transport()? {
            assert_eq!(environment, owner);
        }
    }
    Ok(())
}
