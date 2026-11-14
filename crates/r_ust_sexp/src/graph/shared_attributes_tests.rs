use super::*;
use crate::{ListElement, ListObject, PairListObject, Sexp};

fn attribute(value: Sexp) -> Sexp {
    Sexp::AttributedCharacter {
        values: vec![Some("box".into())],
        attributes: vec![ListElement {
            name: Some("custom".into()),
            value,
        }],
    }
}

#[test]
fn shared_values_attributes_trace_class_and_cons_cycles() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }]),
    )?;
    let head = PairListObject::cons(
        &owner,
        &ListElement {
            name: None,
            value: attribute(Sexp::ListObject(value.clone())),
        },
        None,
    )?;
    let classed = ListObject::from_value(
        &owner,
        &Sexp::ClassedList {
            elements: vec![],
            class: Box::new(attribute(Sexp::PairListObject(head.clone()))),
            class_first: true,
        },
    )?;
    value.replace_shared(&[(0, Sexp::ListObject(classed.clone()))])?;
    let ids = [value.0.id(), head.0.id(), classed.0.id()];
    let exported = classed.value()?;
    drop(classed);
    drop(head);
    drop(value);
    drop(owner);
    for id in ids {
        assert!(heap.lock().node(id).is_ok());
    }
    let Sexp::ClassedList { class, .. } = &exported else {
        panic!("class")
    };
    let Sexp::AttributedCharacter { attributes, .. } = class.as_ref() else {
        panic!("attributes")
    };
    let Sexp::PairListObject(retained) = &attributes[0].value else {
        panic!("cons")
    };
    assert_eq!(retained.0.id(), ids[1]);
    drop(exported);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    Ok(())
}

#[test]
fn shared_values_attributes_foreign_rejection_and_cons_copy_are_atomic() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let other_heap = Arc::new(Heap::default());
    let other = other_heap.session();
    let tail = PairListObject::cons(
        &owner,
        &ListElement {
            name: None,
            value: Sexp::Nil,
        },
        None,
    )?;
    let head = PairListObject::cons(
        &owner,
        &ListElement {
            name: Some("a".into()),
            value: Sexp::Nil,
        },
        Some(&tail),
    )?;
    let before = head.elements()?;
    let count = || {
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count()
    };
    let live = count();
    let incoming = vec![ListElement {
        name: None,
        value: attribute(Sexp::Environment(other)),
    }];
    assert_eq!(head.copy_with(&incoming), Err(GraphError::ForeignHeap));
    assert_eq!(count(), live);
    assert_eq!(head.elements()?, before);
    let copied = head.copy_with(&before)?.ok_or(GraphError::StaleNode)?;
    assert!(!copied.same_object(&head));
    assert!(!copied
        .cdr()?
        .ok_or(GraphError::StaleNode)?
        .same_object(&tail));
    assert_eq!(copied.elements()?, before);
    assert_eq!(head.copy_with(&[])?, None);
    Ok(())
}
