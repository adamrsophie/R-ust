use super::*;
use crate::{Environment, GraphError, ListElement, ListObject, Sexp};

fn named(elements: Vec<Sexp>) -> Sexp {
    Sexp::list_with_metadata(
        elements
            .into_iter()
            .map(|value| ListElement { name: None, value })
            .collect(),
        true,
        None,
    )
}

#[test]
fn named_list_projection_copy_and_shared_replacement_preserve_names_presence(
) -> Result<(), GraphError> {
    let owner = Environment::global();
    for value in [named(vec![]), named(vec![Sexp::Integer(vec![Some(1)])])] {
        let object = ListObject::from_value(&owner, &value)?;
        assert_eq!(object.value()?, value);
        let copy = object.copy_with(&value)?;
        assert!(!object.same_object(&copy));
        assert_eq!(copy.value()?, value);
        if !value
            .ordinary_list()
            .ok_or(GraphError::WrongKind)?
            .elements
            .is_empty()
        {
            object.replace_shared(&[(0, Sexp::Integer(vec![Some(2)]))])?;
            assert_eq!(object.value()?, named(vec![Sexp::Integer(vec![Some(2)])]));
            assert_eq!(copy.value()?, value);
        }
    }
    Ok(())
}

#[test]
fn named_list_edges_reject_foreign_heap_atomically() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let foreign = Environment::global();
    let count = || {
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count()
    };
    let before = count();
    assert_eq!(
        ListObject::from_value(&owner, &named(vec![Sexp::Environment(foreign.clone())])),
        Err(GraphError::ForeignHeap)
    );
    let class_edge = Sexp::NamedList {
        elements: vec![],
        class: Some(Box::new(Sexp::Environment(foreign))),
        class_first: false,
    };
    assert_eq!(
        ListObject::from_value(&owner, &class_edge),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(count(), before);
    Ok(())
}

#[test]
fn named_list_cycles_and_class_edges_release_after_final_export() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let weak = Arc::downgrade(&heap);
    let owner = heap.session();
    let class_owner = owner.child()?;
    let value = Sexp::NamedList {
        elements: vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }],
        class: Some(Box::new(Sexp::Environment(class_owner.clone()))),
        class_first: true,
    };
    let object = ListObject::from_value(&owner, &value)?;
    object.replace_shared(&[(0, Sexp::ListObject(object.clone()))])?;
    owner.insert("x".into(), &Sexp::ListObject(object.clone()))?;
    let ids = [owner.0.id(), class_owner.0.id(), object.0.id()];
    let exported = owner.get_local("x")?;
    drop(value);
    drop(object);
    drop(class_owner);
    drop(owner);
    assert!(heap.lock().node(ids[1]).is_ok());
    assert!(heap.lock().node(ids[2]).is_ok());
    let Some(Sexp::ListObject(export)) = &exported else {
        panic!("export");
    };
    let projection = export.value()?;
    assert!(
        projection
            .ordinary_list()
            .ok_or(GraphError::WrongKind)?
            .names_present
    );
    drop(projection);
    drop(exported);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    drop(heap);
    assert!(weak.upgrade().is_none());
    Ok(())
}
