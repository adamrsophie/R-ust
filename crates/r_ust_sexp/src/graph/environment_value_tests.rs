use super::*;
use crate::{
    CallArgument, ClosureEnvironment, EvaluationOrigin, Force, FunctionParameter, ListElement,
    PromiseState, Sexp,
};

fn wrappers(value: &Sexp) -> Vec<Sexp> {
    let element = ListElement {
        name: Some("e".into()),
        value: value.clone(),
    };
    vec![
        value.clone(),
        Sexp::Vector(vec![value.clone()]),
        Sexp::List(vec![element.clone()]),
        Sexp::pairlist(vec![element.clone()]),
        Sexp::Dots(vec![element.clone()]),
        Sexp::DataFrame {
            columns: vec![element],
            nrow: 1,
        },
        Sexp::Matrix {
            data: Box::new(value.clone()),
            nrow: 1,
            ncol: 1,
        },
        Sexp::Call {
            function: Box::new(value.clone()),
            arguments: vec![],
        },
        Sexp::Call {
            function: Box::new(Sexp::Symbol("f".into())),
            arguments: vec![CallArgument::positional(value.clone())],
        },
        Sexp::Function {
            parameters: vec![FunctionParameter {
                name: "x".into(),
                default: Some(value.clone()),
            }],
            body: Box::new(value.clone()),
        },
        Sexp::Closure {
            parameters: vec![FunctionParameter {
                name: "x".into(),
                default: Some(value.clone()),
            }],
            body: Box::new(value.clone()),
            environment: ClosureEnvironment::Base,
        },
    ]
}

#[test]
fn environment_value_nested_edges_export_identity_and_final_collection() -> Result<(), GraphError> {
    for index in 0..11 {
        let heap = Arc::new(Heap::default());
        let weak = Arc::downgrade(&heap);
        let holder = heap.session();
        let frame = heap.session();
        let id = frame.0.id();
        let identity = frame.display_identity()?;
        let value = Sexp::Environment(frame.clone());
        frame.insert("self".into(), &value)?;
        frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
        let wrapped = wrappers(&value).remove(index);
        holder.insert("stored".into(), &wrapped)?;
        {
            let store = heap.lock();
            let stored = &store.frame(holder.0.id())?.bindings["stored"];
            assert!(
                !stored.edges.is_empty(),
                "environment edge omitted in wrapper {index}"
            );
        }
        drop(wrapped);
        drop(value);
        drop(frame);
        assert!(heap.lock().node(id).is_ok());
        let exported = holder.get_local("stored")?.ok_or(GraphError::StaleNode)?;
        let alias = exported.clone();
        drop(holder);
        drop(exported);
        let consumer = heap.session();
        consumer.insert("alias".into(), &alias)?;
        let promoted = {
            let mut staging = Vec::new();
            let mut store = heap.lock();
            super::super::Environment(store.root(&heap, id, &mut staging))
        };
        assert_eq!(promoted.display_identity()?, identity);
        assert_eq!(
            promoted.get_local("self")?,
            Some(Sexp::Environment(promoted.clone()))
        );
        promoted.insert("marker".into(), &Sexp::Integer(vec![Some(73)]))?;
        let Sexp::Environment(back) = promoted.get_local("self")?.ok_or(GraphError::StaleNode)?
        else {
            panic!("environment")
        };
        assert_eq!(
            back.get_local("marker")?,
            Some(Sexp::Integer(vec![Some(73)]))
        );
        drop(back);
        drop(promoted);
        drop(consumer);
        assert!(heap.lock().node(id).is_ok());
        drop(alias);
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
        assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
        drop(heap);
        assert!(weak.upgrade().is_none());
    }
    Ok(())
}

#[test]
fn environment_value_pending_and_cached_promises() -> Result<(), GraphError> {
    for cached in [false, true] {
        let heap = Arc::new(Heap::default());
        let owner = heap.session();
        let frame = heap.session();
        let id = frame.0.id();
        let expression = Sexp::Environment(frame.clone());
        let promise = PromiseState::new(&expression, &owner, EvaluationOrigin::Global, true)?;
        let pid = promise.0.id();
        if cached {
            let Force::Evaluate(lease) = promise.begin_force()? else {
                panic!("pending")
            };
            lease.complete(&expression)?;
        }
        owner.insert(
            "p".into(),
            &Sexp::Promise {
                state: promise.clone(),
            },
        )?;
        frame.insert("back".into(), &Sexp::Environment(owner.clone()))?;
        drop(expression);
        drop(frame);
        drop(promise);
        let exported = owner.get_local("p")?.ok_or(GraphError::StaleNode)?;
        drop(owner);
        let Sexp::Promise { state } = &exported else {
            panic!("promise")
        };
        match state.begin_force()? {
            Force::Cached(value) => {
                assert!(cached);
                assert!(matches!(value, Sexp::Environment(_)));
            }
            Force::Evaluate(lease) => {
                assert!(!cached);
                assert!(matches!(lease.expression, Sexp::Environment(_)));
            }
        }
        assert!(heap.lock().node(id).is_ok());
        drop(exported);
        for id in [id, pid] {
            assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
        }
        assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
    }
    Ok(())
}

#[test]
fn environment_value_foreign_heap_is_transactional() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let foreign = Arc::new(Heap::default());
    let owner = heap.session();
    let local = heap.session();
    let other = foreign.session();
    let id = other.0.id();
    owner.insert("x".into(), &Sexp::Integer(vec![Some(9)]))?;
    let value = Sexp::Vector(vec![Sexp::Environment(local), Sexp::Environment(other)]);
    assert_eq!(
        owner.insert("x".into(), &value),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(owner.get_local("x")?, Some(Sexp::Integer(vec![Some(9)])));
    drop(value);
    assert!(matches!(
        foreign.lock().node(id),
        Err(GraphError::StaleNode)
    ));
    drop(owner);
    assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
    Ok(())
}
