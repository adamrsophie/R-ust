use super::{heap::Heap, GraphError};
use crate::{ClosureEnvironment, EvaluationOrigin, Force, ListElement, PromiseState, Sexp};
use std::sync::Arc;

fn closure(payload: Sexp) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Nil),
        environment: ClosureEnvironment::Base,
        attributes: vec![ListElement {
            name: Some("srcref".into()),
            value: payload,
        }],
    }
}

#[test]
fn closure_attributes_trace_export_clone_cycle_and_final_release() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let weak = Arc::downgrade(&heap);
    let holder = heap.session();
    let frame = heap.session();
    let frame_id = frame.0.id();
    frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
    let pending = PromiseState::new(
        &Sexp::Symbol("never_force".into()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )?;
    let pending_id = pending.0.id();
    let nested = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Symbol("marker".into())),
        environment: ClosureEnvironment::Captured(frame.clone()),
        attributes: vec![],
    };
    let value = closure(Sexp::List(vec![
        ListElement {
            name: Some("environment".into()),
            value: Sexp::Environment(frame.clone()),
        },
        ListElement {
            name: Some("callback".into()),
            value: nested,
        },
        ListElement {
            name: Some("pending".into()),
            value: Sexp::Promise { state: pending },
        },
    ]));
    holder.insert("x".into(), &value)?;
    frame.insert("cycle".into(), &value)?;
    assert_eq!(
        heap.lock().frame(holder.0.id())?.bindings["x"].edges.len(),
        3
    );
    drop(value);
    drop(frame);
    let exported = holder.get_local("x")?.ok_or(GraphError::StaleNode)?;
    let alias = exported.clone();
    drop(holder);
    drop(exported);
    assert!(heap.lock().node(frame_id).is_ok());
    assert!(heap.lock().node(pending_id).is_ok());
    let Sexp::Closure { attributes, .. } = &alias else {
        panic!("closure lost")
    };
    let Sexp::List(payload) = &attributes[0].value else {
        panic!("payload lost")
    };
    let Sexp::Environment(retained) = &payload[0].value else {
        panic!("environment lost")
    };
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(captured),
        ..
    } = &payload[1].value
    else {
        panic!("callback lost")
    };
    assert_eq!(retained, captured);
    assert_eq!(
        retained.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(42)]))
    );
    let Sexp::Promise { state } = &payload[2].value else {
        panic!("promise lost")
    };
    let Force::Evaluate(lease) = state.begin_force()? else {
        panic!("attribute promise was forced")
    };
    assert!(!lease.interrupted);
    assert_eq!(lease.expression, Sexp::Symbol("never_force".into()));
    assert_eq!(&lease.environment, retained);
    lease.complete(&Sexp::Nil)?;
    drop(alias);
    assert_eq!(
        heap.lock().node(frame_id).err(),
        Some(GraphError::StaleNode)
    );
    assert_eq!(
        heap.lock().node(pending_id).err(),
        Some(GraphError::StaleNode)
    );
    drop(heap);
    assert!(weak.upgrade().is_none());
    Ok(())
}

#[test]
fn closure_attribute_foreign_import_has_no_publication() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let foreign = Arc::new(Heap::default());
    let owner = heap.session();
    let local = heap.session();
    let other = foreign.session();
    let local_id = local.0.id();
    let other_id = other.0.id();
    owner.insert("x".into(), &Sexp::Integer(vec![Some(9)]))?;
    let value = closure(Sexp::Vector(vec![
        Sexp::Environment(local),
        Sexp::Environment(other),
    ]));
    assert_eq!(owner.validate_import(&value), Err(GraphError::ForeignHeap));
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
        foreign.lock().node(other_id).err(),
        Some(GraphError::StaleNode)
    );
    Ok(())
}
