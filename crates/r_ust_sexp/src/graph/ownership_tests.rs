use super::*;
use crate::{ClosureEnvironment, Environment, EvaluationOrigin, Force, PromiseState, Sexp};

fn closure(frame: &Environment) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Nil),
        environment: ClosureEnvironment::Captured(frame.clone()),
    }
}

#[test]
fn cycles_aliases_and_final_root_release() {
    let frame = Environment::global();
    let child = frame.child().unwrap();
    let heap = Arc::clone(frame.0.heap().unwrap());
    let ids = [frame.0.id(), child.0.id()];
    frame.insert("child".into(), &closure(&child)).unwrap();
    child.insert("self".into(), &closure(&child)).unwrap();
    let exported = frame.get("child").unwrap().unwrap();
    let alias = exported.clone();
    drop(exported);
    drop(frame);
    drop(child);
    for id in ids {
        assert!(heap.lock().node(id).is_ok());
    }
    drop(alias);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
}

#[test]
fn promise_cache_cycles_are_traced_and_success_releases_capture() {
    let frame = Environment::global();
    let cache_frame = Environment::global();
    let heap = Arc::clone(frame.0.heap().unwrap());
    let promise = PromiseState::new(&Sexp::Nil, &frame, EvaluationOrigin::Global, true).unwrap();
    let ids = [frame.0.id(), cache_frame.0.id(), promise.0.id()];
    cache_frame
        .insert(
            "promise".into(),
            &Sexp::Promise {
                state: promise.clone(),
            },
        )
        .unwrap();
    let Force::Evaluate(lease) = promise.begin_force().unwrap() else {
        panic!()
    };
    assert!(matches!(
        promise.begin_force(),
        Err(GraphError::PromiseUnderEvaluation)
    ));
    lease.complete(&closure(&cache_frame)).unwrap();
    drop(frame);
    assert!(matches!(
        heap.lock().node(ids[0]),
        Err(GraphError::StaleNode)
    ));
    drop(cache_frame);
    let Force::Cached(exported) = promise.begin_force().unwrap() else {
        panic!()
    };
    drop(promise);
    for id in &ids[1..] {
        assert!(heap.lock().node(*id).is_ok());
    }
    assert_eq!(exported, exported.clone());
    assert!(format!("{exported:?}").contains("Identity"));
    drop(exported);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
}

#[test]
fn interrupted_promise_retains_environment_until_success() {
    let frame = Environment::global();
    let heap = Arc::clone(frame.0.heap().unwrap());
    let id = frame.0.id();
    frame
        .insert("x".into(), &Sexp::Integer(vec![Some(9)]))
        .unwrap();
    let promise = PromiseState::new(
        &Sexp::Symbol("x".into()),
        &frame,
        EvaluationOrigin::Global,
        false,
    )
    .unwrap();
    drop(frame);
    let Force::Evaluate(lease) = promise.begin_force().unwrap() else {
        panic!()
    };
    assert!(!lease.interrupted);
    drop(lease);
    let Force::Evaluate(lease) = promise.begin_force().unwrap() else {
        panic!()
    };
    assert!(lease.interrupted);
    assert_eq!(
        lease.environment.get("x").unwrap(),
        Some(Sexp::Integer(vec![Some(9)]))
    );
    lease.complete(&Sexp::Nil).unwrap();
    assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
}

#[test]
fn generation_reuse_rejects_stale_ids_and_retires_overflow() {
    let mut store = Store::default();
    let frame = || {
        Node::Frame(Frame {
            bindings: HashMap::new(),
            parent: None,
            global: None,
        })
    };
    let old = store.insert(frame());
    store.collect();
    let new = store.insert(frame());
    assert_eq!(old.slot, new.slot);
    assert_ne!(old.generation, new.generation);
    assert!(matches!(store.node(old), Err(GraphError::StaleNode)));
    store.slots[new.slot].generation = u64::MAX;
    store.collect();
    let next = store.insert(frame());
    assert_ne!(next.slot, new.slot);
}

#[test]
fn iterative_collection_reclaims_deep_graph_and_transient_frames() {
    let heap = Arc::new(Heap::default());
    let mut staging = Vec::new();
    let root = {
        let mut store = heap.lock();
        let mut parent = None;
        for _ in 0..20_000 {
            parent = Some(store.insert(Node::Frame(Frame {
                bindings: HashMap::new(),
                parent,
                global: None,
            })));
        }
        store.root(&heap, parent.unwrap(), &mut staging)
    };
    drop(staging);
    assert_eq!(
        heap.lock()
            .slots
            .iter()
            .filter(|s| s.node.is_some())
            .count(),
        20_000
    );
    drop(root);
    assert_eq!(
        heap.lock()
            .slots
            .iter()
            .filter(|s| s.node.is_some())
            .count(),
        0
    );
    let frame = super::super::Environment(heap.insert(Node::Frame(Frame {
        bindings: HashMap::new(),
        parent: None,
        global: None,
    })));
    for _ in 0..10_000 {
        drop(frame.child().unwrap());
    }
    let store = heap.lock();
    let live: Vec<_> = store
        .slots
        .iter()
        .enumerate()
        .filter(|(_, s)| s.node.is_some())
        .map(|(slot, s)| NodeId {
            slot,
            generation: s.generation,
        })
        .collect();
    assert_eq!(live, [frame.0.id()]);
}

#[test]
fn final_host_root_drops_heap_and_controlled_poison_recovers() {
    let heap = Arc::new(Heap::default());
    let frame = super::super::Environment(heap.insert(Node::Frame(Frame {
        bindings: HashMap::new(),
        parent: None,
        global: None,
    })));
    let weak = Arc::downgrade(&heap);
    frame.insert("self".into(), &closure(&frame)).unwrap();
    let mut staging = Vec::new();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut guard = heap.lock();
        let _exported = guard.root(&heap, frame.0.id(), &mut staging);
        panic!("controlled poison");
    }));
    drop(staging);
    assert!(result.is_err());
    frame
        .insert("x".into(), &Sexp::Integer(vec![Some(7)]))
        .unwrap();
    assert_eq!(frame.get("x").unwrap(), Some(Sexp::Integer(vec![Some(7)])));
    drop(heap);
    drop(frame);
    assert!(weak.upgrade().is_none());
}

#[test]
fn promise_expression_edges_and_cross_runtime_cycles() {
    let left = Environment::global();
    let right = Environment::global();
    let heap = Arc::clone(left.0.heap().unwrap());
    let promise =
        PromiseState::new(&closure(&right), &left, EvaluationOrigin::Global, true).unwrap();
    let ids = [left.0.id(), right.0.id(), promise.0.id()];
    right.insert("left".into(), &closure(&left)).unwrap();
    left.insert(
        "promise".into(),
        &Sexp::Promise {
            state: promise.clone(),
        },
    )
    .unwrap();
    drop(left);
    drop(right);
    let exported = promise.expression().unwrap();
    drop(promise);
    for id in ids {
        assert!(heap.lock().node(id).is_ok());
    }
    drop(exported);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
}
