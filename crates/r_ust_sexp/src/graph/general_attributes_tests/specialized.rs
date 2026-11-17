use super::*;

#[test]
fn general_attributes_factor_names_and_custom_metadata_survive_class_removal() -> Result {
    let owner = Environment::global();
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(1), Some(2)]),
        attributes: vec![
            attribute(
                "levels",
                Sexp::String(vec![Some("a".into()), Some("b".into())]),
            ),
            attribute("class", text("factor")),
            attribute("names", Sexp::String(vec![None, Some("".into())])),
            attribute("note", text("kept")),
        ],
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let out = value.without_class()?;
    let Sexp::AttributedAtomic {
        values,
        mut attributes,
    } = raw.clone()
    else {
        panic!("atomic")
    };
    attributes.remove(1);
    assert_eq!(
        out.snapshot()?,
        Sexp::AttributedAtomic { values, attributes }
    );
    assert_eq!(value.snapshot()?, raw);
    Ok(())
}

#[test]
fn general_attributes_matrix_dimnames_custom_and_explicit_class_keep_children() -> Result {
    let owner = Environment::global();
    let dimnames = Sexp::Vector(vec![
        Sexp::String(vec![Some("a".into()), Some("b".into())]),
        Sexp::String(vec![Some("c".into()), Some("d".into())]),
    ]);
    let dimensions = Sexp::Integer(vec![Some(2), Some(2)]);
    for classed in [false, true] {
        let mut attributes = vec![
            attribute("dim", dimensions.clone()),
            attribute("dimnames", dimnames.clone()),
            attribute("note", text("kept")),
        ];
        if classed {
            attributes.push(attribute(
                "class",
                Sexp::String(vec![
                    Some("custom".into()),
                    Some("matrix".into()),
                    Some("array".into()),
                ]),
            ));
        }
        let raw = Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(2), Some(3), Some(4)]),
            attributes,
        };
        let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
        let before = value.view()?.children[0].view()?.children;
        let out = value.without_class()?;
        assert_eq!(value.same_object(&out), !classed);
        assert!(!out.view()?.header.is_object());
        let after = out.view()?.children[0].view()?.children;
        for index in 0..3 {
            assert!(before[index].same_object(&after[index]));
        }
        assert_eq!(
            snapshots(&out.attribute_values()?)?,
            vec![
                attribute("dim", dimensions.clone()),
                attribute("dimnames", dimnames.clone()),
                attribute("note", text("kept")),
            ]
        );
        assert_eq!(value.snapshot()?, raw);
    }
    Ok(())
}
