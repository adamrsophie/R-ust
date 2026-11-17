use super::*;

#[test]
fn general_attributes_invalid_names_class_dimensions_and_rows_publish_nothing() -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let invalid = vec![
        vec![attribute("names", Sexp::String(vec![]))],
        vec![attribute("class", integer(1))],
        vec![attribute("class", text("factor"))],
        vec![attribute("class", Sexp::String(vec![]))],
        vec![attribute("note", integer(1)), attribute("note", integer(2))],
        vec![ListElement {
            name: None,
            value: integer(1),
        }],
        vec![attribute("", integer(1))],
        vec![attribute("dim", Sexp::Integer(vec![Some(2), Some(2)]))],
        vec![attribute("dim", Sexp::Integer(vec![None]))],
        vec![attribute("dim", Sexp::Integer(vec![Some(-1)]))],
        vec![attribute("dimnames", Sexp::Vector(vec![text("x")]))],
        vec![
            attribute("dim", Sexp::Integer(vec![Some(1)])),
            attribute("dimnames", Sexp::Vector(vec![Sexp::String(vec![])])),
        ],
        vec![attribute("row.names", Sexp::Real(vec![1.0]))],
        vec![attribute("row.names", Sexp::Integer(vec![None, None]))],
    ];
    let count = || {
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count()
    };
    for attributes in invalid {
        let before = count();
        assert_eq!(
            ValueObject::import(
                &owner,
                &vector(vec![integer(1)], attributes),
                ValueImport::Evaluated
            ),
            Err(GraphError::WrongKind)
        );
        assert_eq!(count(), before);
    }
    Ok(())
}

#[test]
fn general_attributes_whole_batch_foreign_attribute_rejection_keeps_valid_prefix_and_counts(
) -> Result {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let foreign_heap = Arc::new(Heap::default());
    let foreign = foreign_heap.session();
    let child = ValueObject::import(&owner, &integer(1), ValueImport::Evaluated)?;
    let before = heap
        .lock()
        .slots
        .iter()
        .filter(|slot| slot.node.is_some())
        .count();
    let references = child.semantic_reference_count()?;
    let batch = vec![
        (
            vector(
                vec![child.transport()?],
                vec![attribute("names", text("valid"))],
            ),
            ValueImport::Evaluated,
        ),
        (
            vector(
                vec![child.transport()?],
                vec![attribute("note", Sexp::Environment(foreign.clone()))],
            ),
            ValueImport::Evaluated,
        ),
    ];
    assert_eq!(
        ValueObject::import_many(&owner, &batch),
        Err(GraphError::ForeignHeap)
    );
    assert_eq!(
        heap.lock()
            .slots
            .iter()
            .filter(|slot| slot.node.is_some())
            .count(),
        before
    );
    assert_eq!(child.semantic_reference_count()?, references);
    assert_eq!(child.snapshot()?, integer(1));
    assert!(foreign_heap.lock().node(foreign.0.id()).is_ok());
    Ok(())
}

#[test]
fn general_attributes_compatibility_rejects_invalid_factor_dataframe_and_matrix() -> Result {
    for raw in [
        Sexp::Factor {
            codes: vec![Some(0)],
            levels: vec!["a".into()],
            ordered: false,
        },
        Sexp::Factor {
            codes: vec![Some(2)],
            levels: vec!["a".into()],
            ordered: false,
        },
        Sexp::DataFrame {
            columns: vec![ListElement {
                name: Some("a".into()),
                value: integer(1),
            }],
            nrow: 2,
        },
        Sexp::Matrix {
            data: Box::new(integer(1)),
            nrow: 2,
            ncol: 2,
        },
    ] {
        assert_eq!(raw.attribute_description(), Err(GraphError::WrongKind));
    }
    Ok(())
}
