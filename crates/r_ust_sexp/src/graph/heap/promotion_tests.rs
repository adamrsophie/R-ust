use super::*;
use crate::{Environment, GraphError, ListObject, Sexp};

#[test]
fn repeated_scratch_edges_share_one_root_but_semantic_leases_count_each_slot(
) -> Result<(), GraphError> {
    for semantic in [false, true] {
        let owner = Environment::global();
        let child = ListObject::from_value(&owner, &Sexp::List(Vec::new()))?;
        let value = Sexp::Vector(vec![Sexp::ListObject(child.clone()); 3]);
        let heap = owner.0.heap()?;
        let stored = StoredValue::new(&value, heap)?;
        let mut staging = Vec::new();
        let exported = {
            let mut store = heap.lock();
            stored.export_promoted(
                &mut store,
                &mut RootPromotion::new(heap, &mut staging, semantic),
            )?
        };
        assert_eq!(staging.len(), if semantic { 3 } else { 1 });
        assert_eq!(
            child.semantic_reference_count()?,
            if semantic { 3 } else { 0 }
        );
        drop(staging);
        assert_eq!(
            child.semantic_reference_count()?,
            if semantic { 3 } else { 0 }
        );
        drop(exported);
        assert_eq!(child.semantic_reference_count()?, 0);
        assert_eq!(child.value()?, Sexp::List(Vec::new()));
    }
    Ok(())
}
