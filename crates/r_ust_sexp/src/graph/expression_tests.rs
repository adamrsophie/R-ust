use super::*;
use crate::{CallArgument, ClosureEnvironment, EvaluationOrigin, Force, ListElement, Sexp};

fn expression(value: Sexp) -> Sexp {
    Sexp::Expression {
        elements: vec![ListElement {
            name: Some("leaf".into()),
            value,
        }],
        names_present: true,
    }
}

fn closure(frame: &super::super::Environment) -> Sexp {
    Sexp::Closure {
        attributes: Vec::new(),
        parameters: vec![],
        body: Box::new(Sexp::Symbol("marker".into())),
        environment: ClosureEnvironment::Captured(frame.clone()),
    }
}

#[test]
fn expression_nested_runtime_roots_survive_aliases_and_release_final_cycle(
) -> Result<(), GraphError> {
    for cached in [false, true] {
        let heap = Arc::new(Heap::default());
        let holder = heap.session();
        let frame = heap.session();
        frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
        let promise = crate::PromiseState::new(
            &expression(closure(&frame)),
            &frame,
            EvaluationOrigin::Global,
            true,
        )?;
        let ids = [frame.0.id(), promise.0.id()];
        let tree = expression(expression(Sexp::Call {
            function: Box::new(closure(&frame)),
            arguments: vec![
                CallArgument::positional(Sexp::Environment(frame.clone())),
                CallArgument::positional(Sexp::Promise {
                    state: promise.clone(),
                }),
            ],
        }));
        frame.insert("cycle".into(), &tree)?;
        holder.insert("value".into(), &tree)?;
        if cached {
            let Force::Evaluate(lease) = promise.begin_force()? else {
                panic!("new promise")
            };
            lease.complete(&expression(closure(&frame)))?;
        }
        drop(tree);
        drop(promise);
        drop(frame);
        let exported = holder.get_local("value")?.ok_or(GraphError::StaleNode)?;
        drop(holder);
        assert_access(&exported, cached)?;
        let alias = exported.clone();
        drop(exported);
        assert_access(&alias, cached)?;
        for id in ids {
            assert!(heap.lock().node(id).is_ok());
        }
        drop(alias);
        for id in ids {
            assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
        }
        assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
    }
    Ok(())
}

fn assert_access(value: &Sexp, cached: bool) -> Result<(), GraphError> {
    let Sexp::Expression {
        elements: outer, ..
    } = value
    else {
        panic!("outer")
    };
    let Sexp::Expression { elements, .. } = &outer[0].value else {
        panic!("inner")
    };
    let Sexp::Call {
        function,
        arguments,
    } = &elements[0].value
    else {
        panic!("call")
    };
    let Sexp::Environment(frame) = &arguments[0].value else {
        panic!("frame")
    };
    assert_eq!(
        frame.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(42)]))
    );
    let Sexp::Closure {
        environment: ClosureEnvironment::Captured(captured),
        ..
    } = function.as_ref()
    else {
        panic!("closure")
    };
    assert_eq!(frame, captured);
    let Sexp::Promise { state } = &arguments[1].value else {
        panic!("promise")
    };
    assert!(matches!(state.expression()?, Sexp::Expression { .. }));
    match state.begin_force()? {
        Force::Cached(value) => {
            assert!(cached);
            assert!(matches!(value, Sexp::Expression { .. }));
        }
        Force::Evaluate(lease) => {
            assert!(!cached);
            assert!(matches!(lease.expression, Sexp::Expression { .. }));
        }
    }
    Ok(())
}

#[test]
fn expression_foreign_heap_import_is_transactional() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let other = Arc::new(Heap::default());
    let holder = heap.session();
    let local = heap.session();
    let foreign = other.session();
    let local_id = local.0.id();
    let foreign_id = foreign.0.id();
    holder.insert("value".into(), &Sexp::Integer(vec![Some(7)]))?;
    let tree = expression(Sexp::Call {
        function: Box::new(Sexp::Environment(local)),
        arguments: vec![CallArgument::positional(expression(Sexp::Environment(
            foreign,
        )))],
    });
    assert_eq!(
        holder.insert("value".into(), &tree),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(
        holder.get_local("value")?,
        Some(Sexp::Integer(vec![Some(7)]))
    );
    drop(tree);
    assert!(matches!(
        heap.lock().node(local_id),
        Err(GraphError::StaleNode)
    ));
    assert!(matches!(
        other.lock().node(foreign_id),
        Err(GraphError::StaleNode)
    ));
    Ok(())
}
