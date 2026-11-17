use super::*;

#[test]
fn attribute_replace_plain_aggregate_normalization_keeps_payload_ids_kind_mode_and_gp() -> Result {
    let owner = Environment::global();
    let inputs = [
        Sexp::Vector(vec![integer(1)]),
        Sexp::List(vec![attribute("old", integer(1))]),
        Sexp::Expression {
            elements: vec![attribute("old", integer(1))],
            names_present: true,
        },
        Sexp::DataFrame {
            columns: vec![attribute("old", integer(1))],
            nrow: 1,
        },
    ];
    for input in inputs {
        let value = ValueObject::import(&owner, &input, ValueImport::Code)?;
        set_gp(&value, 0x2040)?;
        let before = value.view()?;
        let copy = value.with_attributes(&[
            attribute("names", text("new")),
            attribute("note", text("kept")),
        ])?;
        let after = copy.view()?;
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.mode, before.mode);
        assert_eq!(after.header.sexptype(), before.header.sexptype());
        assert_eq!(after.header.gp(), 0x2040);
        assert!(!after.header.is_object());
        assert_eq!(after.children.len(), before.children.len() + 1);
        for (old, new) in before.children.iter().zip(&after.children) {
            assert!(old.same_object(new));
        }
        assert_eq!(value.snapshot()?, input);
        assert_eq!(
            snapshots(&copy.attribute_values()?)?,
            vec![
                attribute("names", text("new")),
                attribute("note", text("kept"))
            ]
        );
    }
    Ok(())
}

#[test]
fn attribute_replace_plain_atomic_and_factor_normalization_preserves_values_and_levels() -> Result {
    let owner = Environment::global();
    let inputs = [
        Sexp::Logical(vec![None]),
        integer(1),
        Sexp::Real(vec![2.0]),
        text("a"),
        Sexp::Factor {
            codes: vec![Some(1)],
            levels: vec!["a".into()],
            ordered: true,
        },
    ];
    for input in inputs {
        let value = ValueObject::import(&owner, &input, ValueImport::Evaluated)?;
        set_gp(&value, 0x2040)?;
        let before = value.view()?;
        let mut attrs = value.attribute_values()?;
        attrs.push(attribute("names", text("x")));
        let copy = value.with_attributes(&attrs)?;
        let after = copy.view()?;
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.mode, before.mode);
        assert_eq!(after.header.sexptype(), before.header.sexptype());
        assert_eq!(after.header.gp(), 0x2040);
        let mut expected = input
            .attribute_description()?
            .ok_or(GraphError::WrongKind)?;
        let Sexp::AttributedAtomic { attributes, .. } = &mut expected else {
            panic!("atomic")
        };
        attributes.push(attribute("names", text("x")));
        assert_eq!(copy.snapshot()?, expected);
        assert_eq!(value.snapshot()?, input);
    }
    Ok(())
}

#[test]
fn attribute_replace_nonfinite_payload_and_attribute_bits_stay_exact() -> Result {
    let owner = Environment::global();
    let numbers = vec![
        crate::NA_REAL,
        f64::NAN,
        f64::from_bits(0x7ff8_0000_0000_0001),
        -0.0,
        0.0,
    ];
    for input in [
        Sexp::Real(numbers.clone()),
        Sexp::AttributedAtomic {
            values: AtomicVector::Real(numbers.clone()),
            attributes: vec![],
        },
    ] {
        let value = ValueObject::import(&owner, &input, ValueImport::Evaluated)?;
        let attrs = vec![attribute("note", Sexp::Real(numbers.clone()))];
        let copy = value.with_attributes(&attrs)?;
        let expected = Sexp::AttributedAtomic {
            values: AtomicVector::Real(numbers.clone()),
            attributes: attrs,
        };
        assert!(crate::graph::values::same_snapshot(
            &copy.snapshot()?,
            &expected
        )?);
        assert!(crate::graph::values::same_snapshot(
            &value.snapshot()?,
            &input
        )?);
        let Sexp::AttributedAtomic {
            values: AtomicVector::Real(actual),
            attributes,
        } = copy.snapshot()?
        else {
            panic!("real")
        };
        assert_eq!(
            actual.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            numbers.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
        let Sexp::Real(metadata) = &attributes[0].value else {
            panic!("real metadata")
        };
        assert_eq!(
            metadata.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            numbers.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
    }
    Ok(())
}

#[test]
fn attribute_replace_dataframe_retains_compact_rows_when_supplied_in_replacement() -> Result {
    let owner = Environment::global();
    let raw = Sexp::DataFrame {
        columns: vec![attribute("a", integer(1))],
        nrow: 1,
    };
    let value = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let mut attributes = value.attribute_values()?;
    attributes.push(attribute("note", text("kept")));
    let copy = value.with_attributes(&attributes)?;
    assert!(copy.view()?.header.is_object());
    assert_eq!(copy.view()?.kind, value.view()?.kind);
    assert!(copy.view()?.children[0].same_object(&value.view()?.children[0]));
    let stored = snapshots(&copy.attribute_values()?)?;
    assert_eq!(
        stored,
        vec![
            attribute("names", text("a")),
            attribute("class", text("data.frame")),
            attribute("row.names", Sexp::Integer(vec![None, Some(-1)])),
            attribute("note", text("kept"))
        ]
    );
    assert_eq!(value.snapshot()?, raw);
    Ok(())
}
