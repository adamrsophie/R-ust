use super::*;

#[test]
fn inverse_run_length_discards_names_custom_attributes_and_dimensions() -> Result<(), EvalError> {
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(7), Some(8)]),
        attributes: vec![
            ListElement {
                name: Some("names".into()),
                value: Sexp::String(vec![Some("a".into()), Some("b".into())]),
            },
            ListElement {
                name: Some("note".into()),
                value: Sexp::Integer(vec![Some(3)]),
            },
            ListElement {
                name: Some("dim".into()),
                value: Sexp::Integer(vec![Some(2)]),
            },
        ],
    };
    for input in variants(&raw)? {
        assert_eq!(
            decoded(Sexp::Integer(vec![Some(2), Some(1)]), input)?,
            Sexp::Integer(vec![Some(7), Some(7), Some(8)])
        );
    }
    Ok(())
}

#[test]
fn inverse_run_length_preserves_factor_storage_and_levels_handle() -> Result<(), EvalError> {
    let owner = Environment::global();
    let levels = ValueObject::import(
        &owner,
        &Sexp::String(vec![Some("a".into()), Some("b".into())]),
        ValueImport::Evaluated,
    )?
    .transport()?;
    let raw = Sexp::AttributedAtomic {
        values: AtomicVector::Integer(vec![Some(2), None, Some(1)]),
        attributes: vec![
            ListElement {
                name: Some("class".into()),
                value: Sexp::String(vec![
                    Some("custom".into()),
                    Some("ordered".into()),
                    Some("factor".into()),
                ]),
            },
            ListElement {
                name: Some("levels".into()),
                value: levels.clone(),
            },
            ListElement {
                name: Some("note".into()),
                value: Sexp::Integer(vec![Some(7)]),
            },
        ],
    };
    for input in variants(&raw)? {
        let result = decoded(Sexp::Integer(vec![Some(2), Some(1), Some(1)]), input)?;
        assert_eq!(
            storage::payload(&result)?,
            Sexp::Integer(vec![Some(2), Some(2), None, Some(1)])
        );
        assert_eq!(
            storage::attributes(&result)?,
            vec![
                ListElement {
                    name: Some("class".into()),
                    value: Sexp::String(vec![Some("ordered".into()), Some("factor".into())])
                },
                ListElement {
                    name: Some("levels".into()),
                    value: levels.clone()
                }
            ]
        );
    }
    Ok(())
}

#[test]
fn inverse_run_length_typed_and_managed_records_retain_named_fields() -> Result<(), EvalError> {
    let raw = Sexp::List(vec![
        ListElement {
            name: Some("lengths".into()),
            value: Sexp::Integer(vec![Some(2)]),
        },
        ListElement {
            name: Some("values".into()),
            value: Sexp::Integer(vec![Some(7)]),
        },
    ]);
    for input in variants(&raw)? {
        let result = RuntimeState::default().inverse_rle(&[EvaluatedArgument {
            name: None,
            value: input,
        }])?;
        assert_eq!(result, Sexp::Integer(vec![Some(7), Some(7)]));
    }
    let owner = Environment::global();
    owner.insert("lengths".into(), &Sexp::Integer(vec![Some(2)]))?;
    owner.insert("values".into(), &Sexp::Integer(vec![Some(7)]))?;
    assert_eq!(
        RuntimeState::default().inverse_rle(&[EvaluatedArgument {
            name: None,
            value: Sexp::Environment(owner)
        }])?,
        Sexp::Integer(vec![Some(7), Some(7)])
    );
    Ok(())
}
