use super::*;
use crate::{AssignmentOperator, ClosureEnvironment, Environment, MemberName, Sexp};

#[derive(Clone, Copy)]
enum Edge {
    Target,
    Value,
}

fn assignment(edge: Edge, frame: &Environment) -> Sexp {
    let (target, value) = match edge {
        Edge::Target => (
            Sexp::Dollar {
                value: Box::new(Sexp::Environment(frame.clone())),
                name: MemberName::Symbol("x".into()),
            },
            Sexp::Nil,
        ),
        Edge::Value => (
            Sexp::Symbol("x".into()),
            Sexp::Closure {
                parameters: vec![],
                body: Box::new(Sexp::Symbol("marker".into())),
                environment: ClosureEnvironment::Captured(frame.clone()),
            },
        ),
    };
    Sexp::MemberAssignment {
        operator: AssignmentOperator::Left,
        target: Box::new(target),
        value: Box::new(value),
    }
}

fn retained_edge(edge: Edge) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let weak = Arc::downgrade(&heap);
    let holder = heap.session();
    let frame = heap.session();
    let id = frame.0.id();
    frame.insert("self".into(), &Sexp::Environment(frame.clone()))?;
    frame.insert("marker".into(), &Sexp::Integer(vec![Some(42)]))?;
    let value = assignment(edge, &frame);
    let stored = StoredValue::new(&value, &heap)?;
    assert_eq!(stored.edges, [id]);
    drop(stored);
    holder.insert("ast".into(), &value)?;
    let edges = heap.lock().frame(holder.0.id())?.bindings["ast"]
        .edges
        .clone();
    assert_eq!(edges, [id]);
    drop(value);
    drop(frame);

    let exported = holder.get_local("ast")?.ok_or(GraphError::StaleNode)?;
    drop(holder);
    let alias = exported.clone();
    drop(exported);
    let promoted = {
        let mut staging = Vec::new();
        let mut store = heap.lock();
        super::super::Environment(store.root(&heap, id, &mut staging))
    };
    assert_eq!(alias, assignment(edge, &promoted));
    assert_eq!(
        promoted.get_local("marker")?,
        Some(Sexp::Integer(vec![Some(42)]))
    );
    drop(promoted);
    assert!(heap.lock().node(id).is_ok());
    drop(alias);
    assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
    drop(heap);
    assert!(weak.upgrade().is_none());
    Ok(())
}

#[test]
fn member_assignment_target_edge_retains_identity_and_collects() -> Result<(), GraphError> {
    retained_edge(Edge::Target)
}

#[test]
fn member_assignment_rhs_edge_retains_identity_and_collects() -> Result<(), GraphError> {
    retained_edge(Edge::Value)
}

#[test]
fn member_assignment_foreign_edges_fail_before_replacing_binding() -> Result<(), GraphError> {
    for edge in [Edge::Target, Edge::Value] {
        let heap = Arc::new(Heap::default());
        let other_heap = Arc::new(Heap::default());
        let holder = heap.session();
        let foreign = other_heap.session();
        holder.insert("ast".into(), &Sexp::Integer(vec![Some(7)]))?;
        assert_eq!(
            holder.insert("ast".into(), &assignment(edge, &foreign)),
            Err(GraphError::ForeignHeap)
        );
        assert_eq!(holder.get_local("ast")?, Some(Sexp::Integer(vec![Some(7)])));
        drop(foreign);
        drop(holder);
        assert!(heap.lock().slots.iter().all(|slot| slot.node.is_none()));
        assert!(other_heap
            .lock()
            .slots
            .iter()
            .all(|slot| slot.node.is_none()));
    }
    Ok(())
}
