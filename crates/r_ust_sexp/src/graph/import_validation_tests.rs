use super::*;
use crate::{ListElement, ListObject, Sexp};

fn live(heap: &Heap) -> usize {
    heap.lock()
        .slots
        .iter()
        .filter(|slot| slot.node.is_some())
        .count()
}

#[test]
fn shared_values_import_validation_has_no_publication_or_node_allocation() -> Result<(), GraphError>
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = ListObject::from_value(&owner, &Sexp::List(vec![]))?;
    value.set_gp_shared(7)?;
    owner.insert("kept".into(), &Sexp::ListObject(value.clone()))?;
    let count = live(&heap);
    let slots = heap.lock().slots.len();
    owner.validate_import(&Sexp::ListObject(value.clone()))?;
    owner.validate_import(&Sexp::Nil)?;
    assert_eq!(live(&heap), count);
    assert_eq!(heap.lock().slots.len(), slots);
    assert_eq!(value.header()?.gp(), 7);
    assert_eq!(owner.get_local("kept")?, Some(Sexp::ListObject(value)));
    assert_eq!(heap.lock().frame(owner.0.id())?.bindings.len(), 1);
    Ok(())
}

#[test]
fn shared_values_import_validation_rejects_foreign_unrooted_stale_owner() -> Result<(), GraphError>
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let foreign_heap = Arc::new(Heap::default());
    let foreign = foreign_heap.session();
    owner.insert("kept".into(), &Sexp::Integer(vec![Some(9)]))?;
    let value = Sexp::ClassedList {
        elements: vec![],
        class: Box::new(Sexp::AttributedCharacter {
            values: vec![Some("box".into())],
            attributes: vec![ListElement {
                name: Some("custom".into()),
                value: Sexp::Environment(foreign),
            }],
        }),
        class_first: true,
    };
    let count = live(&heap);
    assert_eq!(owner.validate_import(&value), Err(GraphError::ForeignHeap));
    assert_eq!(
        owner.validate_import(&Sexp::Environment(super::super::Environment(Link::Edge(
            owner.0.id()
        )))),
        Err(GraphError::Unrooted)
    );
    let stale = heap.session();
    let id = stale.0.id();
    drop(stale);
    let stale = super::super::Environment(Link::Root(Arc::new(RootToken {
        heap: heap.clone(),
        id,
    })));
    assert_eq!(
        stale.validate_import(&Sexp::Nil),
        Err(GraphError::StaleNode)
    );
    let vector = ListObject::from_value(&owner, &Sexp::List(vec![]))?;
    assert_eq!(
        super::super::Environment(vector.0.clone()).validate_import(&Sexp::Nil),
        Err(GraphError::WrongKind)
    );
    drop(vector);
    assert_eq!(live(&heap), count);
    assert_eq!(owner.get_local("kept")?, Some(Sexp::Integer(vec![Some(9)])));
    Ok(())
}
