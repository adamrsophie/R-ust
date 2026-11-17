use super::super::value_objects::ValueObject;
use super::*;
use crate::{AtomicVector, Environment, ListElement, Sexp, SexpType, ValueImport};

fn state(heap: &Heap) -> Vec<(u64, bool, u16)> {
    heap.lock()
        .slots
        .iter()
        .map(|slot| (slot.generation, slot.node.is_some(), slot.references))
        .collect()
}

fn rejects(owner: &Environment, value: Sexp, error: GraphError) {
    let heap = owner.0.heap().unwrap();
    let before = state(heap);
    assert_eq!(
        ValueObject::import_many(
            owner,
            &[
                (
                    Sexp::Vector(vec![Sexp::Integer(vec![Some(1)])]),
                    ValueImport::Evaluated
                ),
                (value, ValueImport::Evaluated),
            ]
        )
        .unwrap_err(),
        error
    );
    assert_eq!(state(heap), before);
}

#[test]
fn aggregate_transport_import_batch_validates_every_shape_and_foreign_handle_before_publication(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let foreign_owner = Arc::new(Heap::default()).session();
    let foreign = ValueObject::import(&foreign_owner, &Sexp::List(vec![]), ValueImport::Evaluated)?;
    rejects(&owner, foreign.transport()?, GraphError::ForeignHeap);
    rejects(
        &owner,
        Sexp::AttributedAtomic {
            values: AtomicVector::String(vec![Some("bad\0text".into())]),
            attributes: vec![],
        },
        GraphError::InvalidCharacter,
    );
    for attributes in [
        vec![ListElement {
            name: None,
            value: Sexp::Nil,
        }],
        vec![ListElement {
            name: Some("names".into()),
            value: Sexp::String(vec![]),
        }],
        vec![ListElement {
            name: Some("class".into()),
            value: Sexp::Integer(vec![Some(1)]),
        }],
        vec![ListElement {
            name: Some("class".into()),
            value: Sexp::String(vec![]),
        }],
        vec![ListElement {
            name: Some("dim".into()),
            value: Sexp::Integer(vec![Some(2), Some(2)]),
        }],
        vec![
            ListElement {
                name: Some("x".into()),
                value: Sexp::Nil,
            },
            ListElement {
                name: Some("x".into()),
                value: Sexp::Nil,
            },
        ],
    ] {
        rejects(
            &owner,
            Sexp::AttributedAtomic {
                values: AtomicVector::Integer(vec![Some(1)]),
                attributes,
            },
            GraphError::WrongKind,
        );
    }
    rejects(
        &owner,
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1)]),
            attributes: vec![ListElement {
                name: Some("custom".into()),
                value: foreign.transport()?,
            }],
        },
        GraphError::ForeignHeap,
    );
    Ok(())
}

#[test]
fn aggregate_transport_stale_wrong_type_and_unrooted_handles_publish_no_prefix(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let value = ValueObject::import(&owner, &Sexp::List(vec![]), ValueImport::Evaluated)?;
    let id = value.0.id();
    let wrong_type = ValueObject(value.0.clone(), SexpType::Real);
    rejects(&owner, Sexp::Managed(wrong_type), GraphError::WrongKind);
    rejects(
        &owner,
        Sexp::Managed(ValueObject(Link::Edge(id), SexpType::Vector)),
        GraphError::Unrooted,
    );
    drop(value);
    let stale = ValueObject(
        Link::Root(Arc::new(RootToken {
            heap: heap.clone(),
            id,
            semantic_owner: false,
        })),
        SexpType::Vector,
    );
    rejects(&owner, Sexp::Managed(stale), GraphError::StaleNode);
    Ok(())
}

#[test]
fn aggregate_transport_explicit_mode_rejects_inline_descriptions_without_allocation(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let before = state(&heap);
    assert_eq!(
        ValueObject::import_many(
            &owner,
            &[
                (Sexp::List(vec![]), ValueImport::Evaluated),
                (Sexp::Symbol("code".into()), ValueImport::Explicit),
            ]
        )
        .unwrap_err(),
        GraphError::WrongKind
    );
    assert_eq!(state(&heap), before);
    let handles = ValueObject::import_many(
        &owner,
        &[
            (Sexp::Integer(vec![Some(1)]), ValueImport::Evaluated),
            (Sexp::Symbol("x".into()), ValueImport::Code),
            (Sexp::Environment(owner.clone()), ValueImport::Explicit),
        ],
    )?;
    assert_eq!(handles[0].view()?.mode, Some(ValueImport::Evaluated));
    assert_eq!(handles[1].view()?.mode, Some(ValueImport::Code));
    assert_eq!(handles[2].view()?.mode, None);
    assert_eq!(handles[2].transport()?, Sexp::Environment(owner));
    Ok(())
}
