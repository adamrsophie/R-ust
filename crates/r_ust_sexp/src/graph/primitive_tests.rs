use super::*;
use crate::{Environment, EvaluationOrigin, Force, PrimitiveId, PromiseState, Sexp};

#[test]
fn inspected_promise_capture_stays_rooted_until_host_release() {
    let frame = Environment::global();
    let heap = Arc::clone(frame.0.heap().unwrap());
    let id = frame.0.id();
    let value = Sexp::Primitive(PrimitiveId::Sum);
    frame.insert("value".into(), &value).unwrap();
    let promise = PromiseState::new(
        &Sexp::Symbol("value".into()),
        &frame,
        EvaluationOrigin::Global,
        true,
    )
    .unwrap();
    let exported = promise.environment().unwrap().unwrap();
    drop(frame);
    let Force::Evaluate(lease) = promise.begin_force().unwrap() else {
        panic!()
    };
    assert!(!lease.interrupted);
    lease.complete(&value).unwrap();
    assert!(promise.environment().unwrap().is_none());
    drop(promise);
    assert_eq!(exported.get("value").unwrap(), Some(value));
    drop(exported);
    assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
}

#[test]
fn exported_primitive_is_a_graph_leaf_after_all_roots_drop() {
    let frame = Environment::global();
    let heap = Arc::clone(frame.0.heap().unwrap());
    let id = frame.0.id();
    frame
        .insert("f".into(), &Sexp::Primitive(PrimitiveId::Missing))
        .unwrap();
    let value = frame.get("f").unwrap();
    drop(frame);
    assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    assert_eq!(value, Some(Sexp::Primitive(PrimitiveId::Missing)));
}
