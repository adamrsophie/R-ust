use super::{heap::Heap, GraphError};
use crate::{ClosureEnvironment, ListElement, Sexp, SexpType};
use std::sync::Arc;

fn attributed(value: Sexp) -> Sexp {
    Sexp::AttributedCharacter {
        values: vec![Some("b".into()), None],
        attributes: vec![ListElement {
            name: Some("custom".into()),
            value,
        }],
    }
}

fn roots(class_storage: bool) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let weak = Arc::downgrade(&heap);
    let holder = heap.session();
    let frame = heap.session();
    let id = frame.0.id();
    frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
    let value = wrap(
        attributed(Sexp::List(vec![
            ListElement {
                name: Some("environment".into()),
                value: Sexp::Environment(frame.clone()),
            },
            ListElement {
                name: Some("callback".into()),
                value: Sexp::Closure {
                    parameters: vec![],
                    body: Box::new(Sexp::Symbol("marker".into())),
                    environment: ClosureEnvironment::Captured(frame.clone()),
                },
            },
        ])),
        class_storage,
    );
    assert_eq!(
        value.sexptype(),
        if class_storage {
            SexpType::Vector
        } else {
            SexpType::String
        }
    );
    if !class_storage {
        assert_eq!(value.character_values().map(<[_]>::len), Some(2));
    }
    holder.insert("x".into(), &value)?;
    frame.insert("cycle".into(), &value)?;
    {
        let store = heap.lock();
        assert_eq!(
            store.frame(holder.0.id())?.bindings["x"].edges.len(),
            2,
            "attribute payload must contribute environment and closure edges"
        );
    }
    drop(value);
    drop(frame);
    assert!(heap.lock().node(id).is_ok());
    let exported = holder.get_local("x")?.ok_or(GraphError::StaleNode)?;
    let alias = exported.clone();
    drop(holder);
    drop(exported);
    assert!(heap.lock().node(id).is_ok());
    let character = match &alias {
        Sexp::ClassedList { class, .. } => class.as_ref(),
        other => other,
    };
    let Sexp::AttributedCharacter { attributes, .. } = character else {
        panic!("attributes lost");
    };
    let Sexp::List(payload) = &attributes[0].value else {
        panic!("payload lost");
    };
    let Sexp::Environment(retained) = &payload[0].value else {
        panic!("environment lost");
    };
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(captured),
        ..
    } = &payload[1].value
    else {
        panic!("callback lost");
    };
    assert_eq!(retained, captured);
    assert_eq!(
        retained.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(42)]))
    );
    drop(alias);
    assert_eq!(
        heap.lock().node(id).err(),
        Some(GraphError::StaleNode),
        "attribute cycle must collect"
    );
    drop(heap);
    assert!(
        weak.upgrade().is_none(),
        "final host root must release heap"
    );
    Ok(())
}

fn foreign(class_storage: bool) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let foreign = Arc::new(Heap::default());
    let owner = heap.session();
    let local = heap.session();
    let other = foreign.session();
    let local_id = local.0.id();
    let foreign_id = other.0.id();
    owner.insert("x".into(), &Sexp::Integer(vec![Some(9)]))?;
    let value = wrap(
        attributed(Sexp::Vector(vec![
            Sexp::Environment(local),
            Sexp::Environment(other),
        ])),
        class_storage,
    );
    assert_eq!(
        owner.insert("x".into(), &value),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(owner.get_local("x")?, Some(Sexp::Integer(vec![Some(9)])));
    drop(value);
    assert_eq!(
        heap.lock().node(local_id).err(),
        Some(GraphError::StaleNode)
    );
    assert_eq!(
        foreign.lock().node(foreign_id).err(),
        Some(GraphError::StaleNode)
    );
    Ok(())
}

fn wrap(value: Sexp, class_storage: bool) -> Sexp {
    if class_storage {
        Sexp::ClassedList {
            elements: vec![],
            class: Box::new(value),
            class_first: true,
        }
    } else {
        value
    }
}

#[test]
fn roots_exports_cycles_and_final_release() -> Result<(), GraphError> {
    roots(false)
}
#[test]
fn foreign_attribute_rejection_preserves_binding() -> Result<(), GraphError> {
    foreign(false)
}
#[test]
fn class_attribute_roots_exports_cycles_and_final_release() -> Result<(), GraphError> {
    roots(true)
}
#[test]
fn foreign_class_attribute_rejection_preserves_binding() -> Result<(), GraphError> {
    foreign(true)
}
