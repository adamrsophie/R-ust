use super::*;
use crate::{AtomicVector, ListElement, Sexp, SexpType, ValueImport, ValueObject};

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}

#[test]
fn aggregate_transport_attributed_integer_keeps_names_class_and_storage_header(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    for (name, text) in [("names", "x"), ("class", "expressionsIndex")] {
        let raw = Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1)]),
            attributes: vec![attribute(name, Sexp::String(vec![Some(text.into())]))],
        };
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        assert_eq!(raw.sexptype(), SexpType::Integer);
        assert_eq!(value.transport()?.sexptype(), SexpType::Integer);
        let view = value.view()?;
        assert_eq!(view.kind, crate::ValueKind::Atomic);
        assert_eq!(view.header.sexptype(), SexpType::Integer);
        assert_eq!(view.header.is_object(), name == "class");
        assert_eq!(view.children.len(), 1);
        assert_eq!(view.children[0].view()?.kind, crate::ValueKind::Attributes);
        assert_eq!(value.snapshot()?, raw);
        let borrow = value.borrow()?;
        let Sexp::AttributedAtomic { values, attributes } = borrow.value() else {
            panic!("atomic projection")
        };
        assert_eq!(values, &AtomicVector::Integer(vec![Some(1)]));
        assert_eq!(attributes[0].name.as_deref(), Some(name));
        let Sexp::Managed(child) = &attributes[0].value else {
            panic!("attribute child identity")
        };
        assert!(child.same_object(&view.children[0].view()?.children[0]));
        assert_eq!(borrow.value().snapshot()?, raw);
        assert_eq!(child.semantic_reference_count()?, 1);
    }
    Ok(())
}

#[test]
fn aggregate_transport_atomic_types_preserve_payload_and_ordered_metadata() -> Result<(), GraphError>
{
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    for payload in [
        AtomicVector::Logical(vec![Some(true), None]),
        AtomicVector::Integer(vec![Some(2), None]),
        AtomicVector::Real(vec![2.5, -0.0]),
        AtomicVector::Raw(vec![0, 255]),
        AtomicVector::String(vec![Some("text".into()), None]),
    ] {
        let attributes = vec![
            attribute("dim", Sexp::Integer(vec![Some(1), Some(2)])),
            attribute(
                "names",
                Sexp::String(vec![Some("a".into()), Some("b".into())]),
            ),
            attribute("custom", Sexp::Environment(owner.clone())),
        ];
        let raw = Sexp::AttributedAtomic {
            values: payload.clone(),
            attributes,
        };
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        assert_eq!(value.view()?.header.sexptype(), payload.sexptype());
        assert_eq!(value.snapshot()?, raw);
        let _lease = value.retain()?;
        let attrs = value.view()?.children[0].clone();
        assert_eq!(attrs.semantic_reference_count()?, 1);
        assert_eq!(value.semantic_reference_count()?, 1);
    }
    Ok(())
}

#[test]
fn aggregate_transport_atomic_nonfinite_payloads_and_attributes_preserve_exact_bits(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let expected = vec![
        crate::NA_REAL,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0001),
        -0.0,
        0.0,
    ];
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Real(expected.clone()),
        attributes: vec![attribute("payload", Sexp::Real(expected.clone()))],
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    for snapshot in [
        value.snapshot()?,
        value.borrow()?.value().snapshot()?,
        value.transport()?.snapshot()?,
    ] {
        let Sexp::AttributedAtomic {
            values: AtomicVector::Real(values),
            attributes,
        } = snapshot
        else {
            panic!("real payload")
        };
        assert_eq!(
            values.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
        let Sexp::Real(attribute) = &attributes[0].value else {
            panic!("real attribute")
        };
        assert_eq!(
            attribute.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
        );
    }
    Ok(())
}

#[test]
fn aggregate_transport_managed_class_attribute_reuses_child_and_sets_object_header(
) -> Result<(), GraphError> {
    let heap = Arc::new(Heap::default());
    let owner = heap.session();
    let class = ValueObject::import(
        &owner,
        &Sexp::String(vec![Some("custom".into())]),
        ValueImport::Evaluated,
    )?;
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1)]),
        attributes: vec![attribute("class", class.transport()?)],
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    assert!(value.view()?.header.is_object());
    assert!(value.view()?.children[0].view()?.children[0].same_object(&class));
    assert_eq!(class.semantic_reference_count()?, 1);
    assert_eq!(value.snapshot()?, raw.snapshot()?);
    Ok(())
}
