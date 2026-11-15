use super::*;
use crate::{ListElement, Sexp, ValueImport, ValueObject};

fn state(heap: &Heap) -> (usize, usize) {
    let store = heap.lock();
    (
        store.slots.len(),
        store
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count(),
    )
}

fn rejects(owner: &crate::Environment, value: Sexp, expected: GraphError) {
    let heap = owner.0.heap().unwrap();
    let before = state(heap);
    assert_eq!(
        ValueObject::import(owner, &value, ValueImport::Evaluated).unwrap_err(),
        expected
    );
    assert_eq!(state(heap), before);
}

#[test]
fn aggregate_invalid_late_children_publish_no_prefix() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    for bad in [
        Sexp::Char(Some("bad\0char".into())),
        Sexp::String(vec![Some("bad\0text".into())]),
        Sexp::Matrix {
            data: Box::new(Sexp::Integer(vec![Some(1)])),
            nrow: 2,
            ncol: 1,
        },
    ] {
        let expected = if matches!(bad, Sexp::Matrix { .. }) {
            GraphError::WrongKind
        } else {
            GraphError::InvalidCharacter
        };
        rejects(
            &owner,
            Sexp::Vector(vec![Sexp::List(vec![]), bad]),
            expected,
        );
    }
    let foreign = Arc::new(Heap::default()).session();
    rejects(
        &owner,
        Sexp::AttributedCharacter {
            values: vec![],
            attributes: vec![ListElement {
                name: Some("bad".into()),
                value: Sexp::Environment(foreign),
            }],
        },
        GraphError::ForeignHeap,
    );
    Ok(())
}

#[test]
fn aggregate_stale_and_wrong_kind_handle_validate_before_allocation() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let stale = heap.session();
    let id = stale.0.id();
    drop(stale);
    let stale = crate::graph::Environment(Link::Root(Arc::new(RootToken {
        heap: heap.clone(),
        id,
        semantic_owner: false,
    })));
    rejects(
        &owner,
        Sexp::Vector(vec![Sexp::List(vec![]), Sexp::Environment(stale)]),
        GraphError::StaleNode,
    );
    let list = crate::ListObject::from_value(&owner, &Sexp::List(vec![]))?;
    rejects(
        &owner,
        Sexp::Environment(crate::graph::Environment(list.0.clone())),
        GraphError::WrongKind,
    );
    rejects(
        &owner,
        Sexp::Environment(crate::graph::Environment(Link::Edge(owner.0.id()))),
        GraphError::Unrooted,
    );
    Ok(())
}

#[test]
fn aggregate_stale_owner_publishes_nothing() {
    let heap = Arc::new(Heap::default());
    let stale = heap.session();
    let id = stale.0.id();
    drop(stale);
    let stale = crate::graph::Environment(Link::Root(Arc::new(RootToken {
        heap,
        id,
        semantic_owner: false,
    })));
    rejects(
        &stale,
        Sexp::Vector(vec![Sexp::List(vec![])]),
        GraphError::StaleNode,
    );
}
