use super::*;
use crate::{ListElement, Sexp};

fn list(owner: &Environment, values: Vec<Sexp>) -> Result<ListObject, GraphError> {
    ListObject::from_value(
        owner,
        &Sexp::List(
            values
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
        ),
    )
}

#[test]
fn unreachable_owner_preserves_history_but_explicit_slot_removal_decrements(
) -> Result<(), GraphError> {
    for explicit_removal in [false, true] {
        let heap = Arc::new(Heap::default());
        let owner = heap.session();
        let child = list(&owner, vec![Sexp::Integer(vec![Some(1)])])?;
        let h = list(&owner, vec![Sexp::ListObject(child.clone())])?;
        let junk = list(&owner, vec![Sexp::ListObject(child.clone())])?;
        let junk_id = junk.0.id();
        assert_eq!(child.semantic_reference_count()?, 2);
        if explicit_removal {
            junk.replace_shared(&[(0, Sexp::Nil)])?;
        }
        drop(junk);
        assert_eq!(heap.lock().node(junk_id).err(), Some(GraphError::StaleNode));
        assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 1);
        assert_eq!(
            child.semantic_reference_count()?,
            if explicit_removal { 1 } else { 2 }
        );
        heap.lock().collect();
        assert_eq!(
            child.semantic_reference_count()?,
            if explicit_removal { 1 } else { 2 }
        );
        let Sexp::ListObject(retained) = h.element(0)? else {
            panic!("child identity lost")
        };
        assert!(retained.same_object(&child));
        let child_id = child.0.id();
        drop(retained);
        drop(child);
        drop(h);
        assert_eq!(
            heap.lock().node(child_id).err(),
            Some(GraphError::StaleNode)
        );
    }
    Ok(())
}

#[test]
fn reclaimed_slot_reuse_starts_fresh_without_erasing_surviving_history() -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let child = list(&owner, vec![])?;
    let dead = list(&owner, vec![Sexp::ListObject(child.clone())])?;
    let dead_id = dead.0.id();
    drop(dead);
    assert_eq!(heap.lock().node(dead_id).err(), Some(GraphError::StaleNode));
    assert_eq!(child.semantic_reference_count()?, 1);
    let fresh = list(&owner, vec![])?;
    assert_eq!(fresh.semantic_reference_count()?, 0);
    assert_eq!(heap.lock().node(dead_id).err(), Some(GraphError::StaleNode));
    assert_eq!(child.semantic_reference_count()?, 1);
    assert_eq!(heap.lock().audit_reference_count(child.0.id())?, 0);
    Ok(())
}
