use super::*;
use crate::{ListElement, RuntimeState};
use r_ust_sexp::{AggregateVector, AtomicVector, ValueImport, ValueObject};

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
fn class() -> ListElement {
    attribute("class", Sexp::String(vec![Some("box".into())]))
}

#[test]
fn unclass_sdk_character_keeps_custom_attributes_in_order_and_original_unchanged(
) -> Result<(), EvalError> {
    let before = attribute("before", Sexp::Integer(vec![Some(1)]));
    let after = attribute("after", Sexp::Integer(vec![Some(2)]));
    let value = Sexp::AttributedCharacter {
        values: vec![Some("a".into()), None],
        attributes: vec![before.clone(), class(), after.clone()],
    };
    assert_eq!(
        remove(&value)?,
        Sexp::AttributedCharacter {
            values: vec![Some("a".into()), None],
            attributes: vec![before, after]
        }
    );
    let Sexp::AttributedCharacter { attributes, .. } = value else {
        panic!("character")
    };
    assert_eq!(attributes.len(), 3);
    assert_eq!(attributes[1], class());
    Ok(())
}

#[test]
fn unclass_sdk_factor_preserves_exact_codes_levels_and_storage() -> Result<(), EvalError> {
    for ordered in [false, true] {
        let value = Sexp::Factor {
            codes: vec![Some(2), None, Some(1)],
            levels: vec!["a".into(), "b".into(), "unused".into()],
            ordered,
        };
        assert_eq!(
            remove(&value)?,
            Sexp::AttributedAtomic {
                values: AtomicVector::Integer(vec![Some(2), None, Some(1)]),
                attributes: vec![attribute(
                    "levels",
                    Sexp::String(vec![
                        Some("a".into()),
                        Some("b".into()),
                        Some("unused".into())
                    ])
                )],
            }
        );
    }
    Ok(())
}

#[test]
fn unclass_sdk_dataframe_keeps_compact_row_names_in_persistent_attributes() -> Result<(), EvalError>
{
    for (data, nrow, rownames) in [
        (vec![Some(1), Some(2)], 2, vec![None, Some(-2)]),
        (vec![], 0, vec![]),
    ] {
        let column = Sexp::Integer(data);
        let value = Sexp::DataFrame {
            columns: vec![attribute("a", column.clone())],
            nrow,
        };
        assert_eq!(
            remove(&value)?,
            Sexp::AttributedAggregate {
                values: AggregateVector::Vector(vec![column]),
                attributes: vec![
                    attribute("names", Sexp::String(vec![Some("a".into())])),
                    attribute("row.names", Sexp::Integer(rownames))
                ],
            }
        );
    }
    Ok(())
}

#[test]
fn unclass_sdk_managed_no_object_preserves_identity_and_reference_count() -> Result<(), EvalError> {
    let runtime = RuntimeState::default();
    let object = ValueObject::import(
        &runtime.current,
        &Sexp::Integer(vec![Some(1)]),
        ValueImport::Evaluated,
    )?;
    let count = object.semantic_reference_count()?;
    let Sexp::Managed(result) = remove(&object.transport()?)? else {
        panic!("managed")
    };
    assert!(result.same_object(&object));
    assert_eq!(object.semantic_reference_count()?, count);
    assert_eq!(
        remove(&Sexp::Environment(runtime.current.clone()))?,
        Sexp::Environment(runtime.current.clone())
    );
    Ok(())
}

#[test]
fn unclass_sdk_managed_copy_keeps_payload_and_nonclass_attribute_identity() -> Result<(), EvalError>
{
    let runtime = RuntimeState::default();
    let child = ValueObject::import(
        &runtime.current,
        &Sexp::Integer(vec![Some(7)]),
        ValueImport::Evaluated,
    )?;
    let metadata = ValueObject::import(
        &runtime.current,
        &Sexp::String(vec![Some("meta".into())]),
        ValueImport::Evaluated,
    )?;
    let object = ValueObject::import(
        &runtime.current,
        &Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![child.transport()?, child.transport()?]),
            attributes: vec![attribute("custom", metadata.transport()?), class()],
        },
        ValueImport::Evaluated,
    )?;
    let Sexp::Managed(result) = remove(&object.transport()?)? else {
        panic!("managed")
    };
    assert!(!result.same_object(&object));
    assert!(object.view()?.header.is_object());
    assert!(!result.view()?.header.is_object());
    let attrs = result.attribute_values()?;
    assert_eq!(attrs.len(), 1);
    let Sexp::Managed(meta) = &attrs[0].value else {
        panic!("attribute identity")
    };
    assert!(meta.same_object(&metadata));
    let borrowed = result.borrow()?;
    let Sexp::AttributedAggregate {
        values: AggregateVector::Vector(values),
        ..
    } = borrowed.value()
    else {
        panic!("aggregate")
    };
    for value in values {
        let Sexp::Managed(value) = value else {
            panic!("child identity")
        };
        assert!(value.same_object(&child));
    }
    drop(borrowed);
    drop(object);
    drop(runtime);
    assert_eq!(
        result.snapshot()?,
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![
                Sexp::Integer(vec![Some(7)]),
                Sexp::Integer(vec![Some(7)])
            ]),
            attributes: vec![attribute("custom", Sexp::String(vec![Some("meta".into())]))],
        }
    );
    Ok(())
}

#[test]
fn unclass_sdk_empty_argument_tag_matches_check1arg() -> Result<(), EvalError> {
    let mut runtime = RuntimeState::default();
    let result = runtime.eval(&Sexp::Call {
        function: Box::new(Sexp::Primitive(r_ust_sexp::PrimitiveId::Unclass)),
        arguments: vec![crate::CallArgument::named("", Sexp::Integer(vec![Some(1)]))],
    })?;
    assert_eq!(result, Sexp::Integer(vec![Some(1)]));
    Ok(())
}

#[test]
fn unclass_sdk_managed_explicit_graph_list_uses_same_copy_semantics() -> Result<(), EvalError> {
    let runtime = RuntimeState::default();
    let raw = Sexp::list_with_metadata(
        vec![attribute("a", Sexp::Integer(vec![Some(1)]))],
        true,
        Some((class().value, false)),
    );
    let list = r_ust_sexp::ListObject::from_value(&runtime.current, &raw)?;
    let handle = ValueObject::import(
        &runtime.current,
        &Sexp::ListObject(list.clone()),
        ValueImport::Explicit,
    )?;
    let Sexp::ListObject(result) = remove(&Sexp::Managed(handle))? else {
        panic!("explicit list")
    };
    assert!(!result.same_object(&list));
    assert_eq!(list.value()?, raw);
    assert!(result
        .value()?
        .ordinary_list()
        .expect("list")
        .class
        .is_none());
    Ok(())
}
