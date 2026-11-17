use super::*;

#[test]
fn inverse_run_length_default_field_matching_keeps_exact_precedence_and_unique_prefixes(
) -> Result<(), EvalError> {
    for fields in [
        vec![("lengthsLong", 2), ("valuesLong", 7)],
        vec![("lengths", 2), ("lengths", 3), ("values", 7)],
    ] {
        let raw = Sexp::List(
            fields
                .into_iter()
                .map(|(name, value)| ListElement {
                    name: Some(name.into()),
                    value: Sexp::Integer(vec![Some(value)]),
                })
                .collect(),
        );
        for input in variants(&raw)? {
            assert_eq!(
                RuntimeState::default().inverse_rle(&[EvaluatedArgument {
                    name: None,
                    value: input
                }])?,
                Sexp::Integer(vec![Some(7), Some(7)])
            );
        }
    }
    let ambiguous = Sexp::List(vec![
        ListElement {
            name: Some("lengthsLong".into()),
            value: Sexp::Integer(vec![Some(2)]),
        },
        ListElement {
            name: Some("lengthsOther".into()),
            value: Sexp::Integer(vec![Some(3)]),
        },
        ListElement {
            name: Some("values".into()),
            value: Sexp::Integer(vec![Some(7)]),
        },
    ]);
    assert!(
        matches!(RuntimeState::default().inverse_rle(&[EvaluatedArgument{name:None,value:ambiguous}]),Err(EvalError::Domain(message)) if message=="invalid 'rle' structure")
    );
    Ok(())
}

#[test]
fn inverse_run_length_expression_call_and_pairlist_records_preserve_fields() -> Result<(), EvalError>
{
    let fields = vec![
        ListElement {
            name: Some("lengths".into()),
            value: Sexp::Integer(vec![Some(2)]),
        },
        ListElement {
            name: Some("values".into()),
            value: Sexp::Integer(vec![Some(7)]),
        },
    ];
    let expression = Sexp::Expression {
        elements: fields.clone(),
        names_present: true,
    };
    let pairlist = Sexp::pairlist(fields.clone());
    let call = Sexp::Call {
        function: Box::new(Sexp::Symbol("never_call".into())),
        arguments: fields
            .into_iter()
            .map(|e| r_ust_sexp::CallArgument {
                name: e.name,
                value: e.value,
            })
            .collect(),
    };
    for raw in [expression, pairlist, call] {
        for input in variants(&raw)? {
            assert_eq!(
                RuntimeState::default().inverse_rle(&[EvaluatedArgument {
                    name: None,
                    value: input
                }])?,
                Sexp::Integer(vec![Some(7), Some(7)])
            );
        }
    }
    Ok(())
}

#[test]
fn run_length_explicit_cons_and_list_handles_keep_payload_roots() -> Result<(), EvalError> {
    let owner = Environment::global();
    let child = ValueObject::import(
        &owner,
        &Sexp::Integer(vec![Some(7)]),
        ValueImport::Evaluated,
    )?
    .transport()?;
    let pair = r_ust_sexp::PairListObject::cons(
        &owner,
        &ListElement {
            name: Some("a".into()),
            value: child.clone(),
        },
        None,
    )?;
    let list = r_ust_sexp::ListObject::from_value(
        &owner,
        &Sexp::List(vec![ListElement {
            name: None,
            value: child.clone(),
        }]),
    )?;
    for input in [Sexp::PairListObject(pair), Sexp::ListObject(list)] {
        let result = encoded(input)?;
        let view = result.ordinary_list().unwrap();
        let plain = storage::payload(&view.elements[1].value)?;
        let Sexp::Vector(values) = plain else {
            panic!("vector payload")
        };
        assert_eq!(values, vec![child.clone()]);
    }
    Ok(())
}
