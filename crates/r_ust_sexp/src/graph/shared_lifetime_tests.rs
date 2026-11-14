use super::*;
use crate::{
    ClosureEnvironment, EvaluationOrigin, ListElement, ListObject, PairListObject, PromiseState,
    Sexp,
};

fn slot(value: Sexp) -> ListElement {
    ListElement { name: None, value }
}

#[test]
fn shared_values_cons_lifetime_closure_promise_and_export() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let weak = Arc::downgrade(&heap);
    let owner = heap.session();
    let capture = owner.child()?;
    let vector = ListObject::from_value(&owner, &Sexp::List(vec![slot(Sexp::Nil)]))?;
    let pending = PromiseState::new(
        &Sexp::ListObject(vector.clone()),
        &capture,
        EvaluationOrigin::Global,
        true,
    )?;
    let callback = Sexp::Closure {
        parameters: vec![],
        body: Box::new(Sexp::Promise {
            state: pending.clone(),
        }),
        environment: ClosureEnvironment::Captured(capture.clone()),
    };
    let head = PairListObject::cons(&owner, &slot(callback), None)?;
    vector.replace_shared(&[(0, Sexp::PairListObject(head.clone()))])?;
    capture.insert("cycle".into(), &Sexp::ListObject(vector.clone()))?;
    owner.insert("head".into(), &Sexp::PairListObject(head.clone()))?;
    let ids = [
        owner.0.id(),
        capture.0.id(),
        vector.0.id(),
        pending.0.id(),
        head.0.id(),
    ];
    let exported = owner.get_local("head")?;
    drop(head);
    drop(pending);
    drop(vector);
    drop(capture);
    drop(owner);
    for id in ids {
        assert!(heap.lock().node(id).is_ok());
    }
    drop(exported);
    for id in ids {
        assert!(matches!(heap.lock().node(id), Err(GraphError::StaleNode)));
    }
    drop(heap);
    assert!(weak.upgrade().is_none());
    Ok(())
}

#[test]
fn shared_values_cons_lifetime_foreign_tail_is_atomic() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let other_heap = Arc::new(Heap::default());
    let other = other_heap.session();
    let foreign = PairListObject::cons(&other, &slot(Sexp::Nil), None)?;
    let count = || {
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count()
    };
    let before = count();
    assert_eq!(
        PairListObject::cons(&owner, &slot(Sexp::Nil), Some(&foreign)),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(count(), before);
    assert_eq!(
        PairListObject::cons(&owner, &slot(Sexp::Environment(other)), None),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(count(), before);
    let wrong_kind = super::super::PairListObject(owner.0.clone());
    assert_eq!(wrong_kind.car(), Err(GraphError::WrongKind));
    assert_eq!(
        PairListObject::cons(&owner, &slot(Sexp::Nil), Some(&wrong_kind)),
        Err(GraphError::WrongKind)
    );
    assert_eq!(count(), before);
    Ok(())
}
