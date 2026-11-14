use super::*;
use crate::{
    CallArgument, ClosureEnvironment, Environment, EvaluationOrigin, Force, ListElement,
    PromiseState, Sexp,
};

fn closure(frame: &Environment) -> Sexp {
    Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Nil),
        environment: ClosureEnvironment::Captured(frame.clone()),
    }
}

fn pair(value: Sexp) -> Sexp {
    Sexp::pairlist(vec![ListElement {
        name: Some("x".into()),
        value,
    }])
}

fn assert_capture_access(value: &Sexp) -> Result<(), GraphError> {
    let Sexp::PairList(elements) = value else {
        panic!("captured pairlist")
    };
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(frame),
        ..
    } = &elements.as_slice()[0].value
    else {
        panic!("captured closure")
    };
    assert_eq!(frame.get("marker")?, Some(Sexp::Integer(vec![Some(42)])));
    Ok(())
}

fn call(value: Sexp, head: bool) -> Sexp {
    if head {
        Sexp::Call {
            function: Box::new(value),
            arguments: vec![],
        }
    } else {
        Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![CallArgument::positional(value)],
        }
    }
}

#[test]
fn canonical_language_pending_and_cached_promises_survive_export_without_forcing(
) -> Result<(), GraphError> {
    for cached in [false, true] {
        for head in [false, true] {
            let container = Environment::global();
            let frame = Environment::global();
            let capture = Environment::global();
            capture.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
            let heap = Arc::clone(container.0.heap()?);
            let expression = pair(closure(&capture));
            let promise = PromiseState::new(&expression, &frame, EvaluationOrigin::Global, true)?;
            let ids = [frame.0.id(), capture.0.id(), promise.0.id()];
            let value = pair(call(
                Sexp::Promise {
                    state: promise.clone(),
                },
                head,
            ));
            capture.insert("back".into(), &value)?;
            container.insert("value".into(), &value)?;
            if cached {
                let Force::Evaluate(lease) = promise.begin_force()? else {
                    panic!("new promise was cached")
                };
                lease.complete(&expression)?;
            }
            drop(expression);
            drop(value);
            drop(promise);
            drop(frame);
            drop(capture);
            let exported = container.get("value")?.ok_or(GraphError::StaleNode)?;
            drop(container);
            let Sexp::PairList(elements) = &exported else {
                panic!("pairlist export")
            };
            let Sexp::Call {
                function,
                arguments,
            } = &elements.as_slice()[0].value
            else {
                panic!("call export")
            };
            let value = if head {
                function.as_ref()
            } else {
                &arguments[0].value
            };
            let Sexp::Promise { state } = value else {
                panic!("promise export")
            };
            match state.begin_force()? {
                Force::Cached(value) => {
                    assert!(cached);
                    assert_capture_access(&value)?;
                }
                Force::Evaluate(lease) => {
                    assert!(!cached);
                    assert!(!lease.interrupted);
                    assert_capture_access(&lease.expression)?;
                }
            }
            assert!(heap.lock().node(ids[1]).is_ok());
            assert!(heap.lock().node(ids[2]).is_ok());
            let alias = exported.clone();
            drop(exported);
            assert!(heap.lock().node(ids[1]).is_ok());
            drop(alias);
            for id in ids {
                assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
            }
        }
    }
    Ok(())
}

#[test]
fn canonical_language_foreign_heap_partial_visit_preserves_binding_and_releases_roots(
) -> Result<(), GraphError> {
    let container = Environment::global();
    let capture = Environment::global();
    let heap = Arc::clone(container.0.heap()?);
    assert!(Arc::ptr_eq(&heap, capture.0.heap()?));
    let foreign_heap = Arc::new(Heap::default());
    let foreign = super::super::Environment(foreign_heap.insert(Node::Frame(Frame {
        bindings: HashMap::new(),
        parent: None,
        global: None,
    })));
    assert!(!Arc::ptr_eq(&heap, foreign.0.heap()?));
    let capture_id = capture.0.id();
    let foreign_id = foreign.0.id();
    capture.insert("self".into(), &closure(&capture))?;
    foreign.insert("self".into(), &closure(&foreign))?;
    container.insert("value".into(), &Sexp::Integer(vec![Some(7)]))?;
    let value = Sexp::Call {
        function: Box::new(pair(closure(&capture))),
        arguments: vec![CallArgument::positional(pair(closure(&foreign)))],
    };
    assert_eq!(
        container.insert("value".into(), &value),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(container.get("value")?, Some(Sexp::Integer(vec![Some(7)])));
    drop(value);
    drop(capture);
    drop(foreign);
    assert!(matches!(
        heap.lock().node(capture_id),
        Err(GraphError::StaleNode)
    ));
    assert!(matches!(
        foreign_heap.lock().node(foreign_id),
        Err(GraphError::StaleNode)
    ));
    assert_eq!(container.get("value")?, Some(Sexp::Integer(vec![Some(7)])));
    Ok(())
}
