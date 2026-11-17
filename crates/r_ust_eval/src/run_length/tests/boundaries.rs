use super::*;

#[test]
fn run_length_atomic_metadata_boundary_rejects_class_dim_and_custom_attributes(
) -> Result<(), EvalError> {
    for (name, value) in [
        ("note", Sexp::Integer(vec![Some(7)])),
        ("class", Sexp::String(vec![Some("custom".into())])),
        ("dim", Sexp::Integer(vec![Some(2)])),
    ] {
        let raw = Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1), Some(1)]),
            attributes: vec![ListElement {
                name: Some(name.into()),
                value,
            }],
        };
        for input in variants(&raw)? {
            assert!(
                matches!(encoded(input),Err(EvalError::Type(message)) if message=="'x' must be a vector of an atomic type")
            );
        }
    }
    for raw in [
        Sexp::Nil,
        Sexp::Environment(Environment::global()),
        Sexp::Char(Some("a".into())),
        Sexp::Symbol("x".into()),
    ] {
        for input in variants(&raw)? {
            assert!(encoded(input).is_err());
        }
    }
    Ok(())
}

#[test]
fn run_length_empty_payload_preserves_managed_identity_and_metadata() -> Result<(), EvalError> {
    for raw in [
        Sexp::Integer(vec![]),
        Sexp::Expression {
            elements: vec![],
            names_present: false,
        },
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![]),
            attributes: vec![ListElement {
                name: Some("names".into()),
                value: Sexp::String(vec![]),
            }],
        },
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![]),
            attributes: vec![ListElement {
                name: Some("note".into()),
                value: Sexp::Integer(vec![Some(7)]),
            }],
        },
    ] {
        for input in variants(&raw)? {
            let result = encoded(input.clone())?;
            let view = result.ordinary_list().unwrap();
            assert_eq!(view.elements[0].value, Sexp::Integer(vec![]));
            assert_eq!(view.elements[1].value, input);
        }
    }
    Ok(())
}

#[test]
fn run_length_single_list_expression_pairlist_accept_but_multiple_elements_error(
) -> Result<(), EvalError> {
    let child = Sexp::Integer(vec![Some(7)]);
    for raw in [
        Sexp::Vector(vec![child.clone()]),
        Sexp::Expression {
            elements: vec![ListElement {
                name: None,
                value: child.clone(),
            }],
            names_present: false,
        },
        Sexp::pairlist(vec![ListElement {
            name: None,
            value: child.clone(),
        }]),
    ] {
        for input in variants(&raw)? {
            let result = encoded(input)?;
            assert_eq!(
                result.ordinary_list().unwrap().elements[0].value,
                Sexp::Integer(vec![Some(1)])
            );
        }
    }
    for input in variants(&Sexp::Vector(vec![child.clone(), child]))? {
        assert!(
            matches!(encoded(input),Err(EvalError::Type(message)) if message=="comparison of these types is not implemented")
        );
    }
    Ok(())
}
