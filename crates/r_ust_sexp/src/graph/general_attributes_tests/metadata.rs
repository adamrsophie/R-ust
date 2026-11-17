use super::*;

#[test]
fn general_attributes_list_names_presence_na_blank_and_order_survive_class_removal() -> Result {
    let owner = Environment::global();
    for first in [false, true] {
        let raw = Sexp::ClassedList {
            elements: vec![
                ListElement {
                    name: None,
                    value: integer(1),
                },
                ListElement {
                    name: Some("".into()),
                    value: integer(2),
                },
                ListElement {
                    name: Some("named".into()),
                    value: integer(3),
                },
            ],
            class: Box::new(text("custom")),
            class_first: first,
        };
        let value = ValueObject::import(&owner, &description(&raw)?, ValueImport::Evaluated)?;
        let attributes = snapshots(&value.attribute_values()?)?;
        assert_eq!(
            attributes
                .iter()
                .map(|a| a.name.as_deref())
                .collect::<Vec<_>>(),
            if first {
                vec![Some("class"), Some("names")]
            } else {
                vec![Some("names"), Some("class")]
            }
        );
        let out = value.without_class()?;
        assert!(!value.same_object(&out));
        assert!(value.view()?.header.is_object());
        assert!(!out.view()?.header.is_object());
        assert_eq!(
            snapshots(&out.attribute_values()?)?,
            vec![attribute(
                "names",
                Sexp::String(vec![None, Some("".into()), Some("named".into())])
            )]
        );
    }
    for named in [false, true] {
        let raw = Sexp::list_with_metadata(vec![], named, Some((text("custom"), false)));
        let out = ValueObject::import(&owner, &description(&raw)?, ValueImport::Evaluated)?
            .without_class()?;
        assert_eq!(
            snapshots(&out.attribute_values()?)?,
            if named {
                vec![attribute("names", Sexp::String(vec![]))]
            } else {
                vec![]
            }
        );
    }
    Ok(())
}

#[test]
fn general_attributes_dataframe_compact_and_zero_rows_keep_storage_order() -> Result {
    let owner = Environment::global();
    for count in [0usize, 3] {
        let raw = Sexp::DataFrame {
            columns: vec![ListElement {
                name: Some("a".into()),
                value: Sexp::Integer(vec![Some(1); count]),
            }],
            nrow: count,
        };
        let value = ValueObject::import(&owner, &description(&raw)?, ValueImport::Evaluated)?;
        let rows = if count == 0 {
            Sexp::Integer(vec![])
        } else {
            Sexp::Integer(vec![None, Some(-3)])
        };
        assert_eq!(
            snapshots(&value.attribute_values()?)?,
            vec![
                attribute("names", text("a")),
                attribute("class", text("data.frame")),
                attribute("row.names", rows.clone())
            ]
        );
        let unclassed = value.without_class()?;
        assert_eq!(
            snapshots(&unclassed.attribute_values()?)?,
            vec![
                attribute("names", text("a")),
                attribute("row.names", rows.clone())
            ]
        );
        assert_eq!(
            crate::expanded_row_names(&rows)?,
            Sexp::Integer(if count == 0 {
                vec![]
            } else {
                vec![Some(1), Some(2), Some(3)]
            })
        );
        assert!(matches!(
            unclassed.snapshot()?,
            Sexp::AttributedAggregate { .. }
        ));
    }
    let empty = description(&Sexp::DataFrame {
        columns: vec![],
        nrow: 0,
    })?;
    let Sexp::AttributedAggregate { attributes, .. } = empty else {
        panic!("aggregate")
    };
    assert_eq!(
        attributes,
        vec![
            attribute("names", Sexp::String(vec![])),
            attribute("row.names", Sexp::Integer(vec![])),
            attribute("class", text("data.frame"))
        ]
    );
    Ok(())
}

#[test]
fn general_attributes_explicit_and_signed_compact_row_names_are_not_overwritten() -> Result {
    let owner = Environment::global();
    for (rows, public) in [
        (
            Sexp::Integer(vec![None, Some(3)]),
            Sexp::Integer(vec![Some(1), Some(2), Some(3)]),
        ),
        (
            Sexp::Integer(vec![None, Some(-3)]),
            Sexp::Integer(vec![Some(1), Some(2), Some(3)]),
        ),
        (
            Sexp::Integer(vec![Some(5), Some(7), Some(9)]),
            Sexp::Integer(vec![Some(5), Some(7), Some(9)]),
        ),
        (
            Sexp::String(vec![Some("r1".into()), Some("r2".into())]),
            Sexp::String(vec![Some("r1".into()), Some("r2".into())]),
        ),
    ] {
        let raw = vector(
            vec![],
            vec![
                attribute("row.names", rows.clone()),
                attribute("class", text("data.frame")),
                attribute("custom", integer(17)),
            ],
        );
        let out = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?.without_class()?;
        assert_eq!(
            snapshots(&out.attribute_values()?)?,
            vec![
                attribute("row.names", rows.clone()),
                attribute("custom", integer(17))
            ]
        );
        assert_eq!(crate::expanded_row_names(&rows)?, public);
    }
    Ok(())
}

#[test]
fn general_attributes_factor_codes_levels_order_and_object_header_are_preserved() -> Result {
    let owner = Environment::global();
    for ordered in [false, true] {
        let raw = Sexp::Factor {
            codes: vec![Some(2), None, Some(1)],
            levels: vec!["a".into(), "b".into(), "unused".into()],
            ordered,
        };
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        assert!(value.view()?.header.is_object());
        let out = value.without_class()?;
        assert!(!out.view()?.header.is_object());
        assert_eq!(
            out.snapshot()?,
            Sexp::AttributedAtomic {
                values: AtomicVector::Integer(vec![Some(2), None, Some(1)]),
                attributes: vec![attribute(
                    "levels",
                    Sexp::String(vec![
                        Some("a".into()),
                        Some("b".into()),
                        Some("unused".into())
                    ])
                )]
            }
        );
        assert_eq!(value.snapshot()?, raw);
    }
    Ok(())
}

#[test]
fn general_attributes_atomic_and_list_matrix_keep_dims_without_object_bit() -> Result {
    let owner = Environment::global();
    for payload in [
        Sexp::Integer(vec![Some(1); 4]),
        Sexp::Vector(vec![integer(1); 4]),
    ] {
        let raw = Sexp::Matrix {
            data: Box::new(payload),
            nrow: 2,
            ncol: 2,
        };
        let value = ValueObject::import(&owner, &description(&raw)?, ValueImport::Evaluated)?;
        assert!(!value.view()?.header.is_object());
        assert!(value.same_object(&value.without_class()?));
        assert_eq!(
            snapshots(&value.attribute_values()?)?,
            vec![attribute("dim", Sexp::Integer(vec![Some(2), Some(2)]))]
        );
    }
    Ok(())
}
