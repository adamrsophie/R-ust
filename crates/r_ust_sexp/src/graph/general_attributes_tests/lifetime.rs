use super::*;

#[test]
fn general_attributes_class_removal_retains_nested_owners_after_origin_drop_then_collects() -> Result
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let child = owner.child()?;
    child.insert("marker".into(), &integer(11))?;
    let value = ValueObject::import(
        &owner,
        &vector(
            vec![Sexp::Environment(child.clone())],
            vec![
                attribute("class", text("custom")),
                attribute("note", Sexp::Environment(child.clone())),
            ],
        ),
        ValueImport::Evaluated,
    )?;
    let unclassed = value.without_class()?;
    child.insert("cycle".into(), &unclassed.transport()?)?;
    let ids = [owner.0.id(), child.0.id(), value.0.id(), unclassed.0.id()];
    drop(value);
    drop(child);
    drop(owner);
    assert!(matches!(
        heap.lock().node(ids[2]),
        Err(GraphError::StaleNode)
    ));
    let borrow = unclassed.borrow()?;
    let Sexp::AttributedAggregate {
        values: AggregateVector::Vector(values),
        attributes,
    } = borrow.value()
    else {
        panic!("aggregate")
    };
    let Sexp::Environment(child) = &values[0] else {
        panic!("payload environment")
    };
    assert_eq!(child.get_local("marker")?, Some(integer(11)));
    assert_eq!(attributes[0].name.as_deref(), Some("note"));
    assert_eq!(attributes[0].value, Sexp::Environment(child.clone()));
    drop(borrow);
    drop(unclassed);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
    Ok(())
}

#[test]
fn general_attributes_pending_noop_alias_and_changed_copy_keep_semantic_history() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = ValueObject::import(
        &owner,
        &vector(vec![integer(1)], vec![attribute("class", text("custom"))]),
        ValueImport::Evaluated,
    )?;
    owner.insert("x".into(), &value.transport()?)?;
    let copy = value.without_class()?;
    let before = value.semantic_reference_count()?;
    assert_eq!(owner.begin_assignment("x")?, Some(false));
    owner.insert("x".into(), &value.transport()?)?;
    assert_eq!(owner.begin_assignment("x")?, Some(true));
    assert!(owner.finish_assignment("x")?);
    assert_eq!(value.semantic_reference_count()?, before);
    assert!(!value.same_object(&copy));
    assert!(copy.same_object(&copy.without_class()?));
    assert!(value.view()?.header.is_object());
    assert!(!copy.view()?.header.is_object());
    Ok(())
}
