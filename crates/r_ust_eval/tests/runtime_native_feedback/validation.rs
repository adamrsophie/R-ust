use super::*;
use r_ust_sexp::GraphError;

fn containers(child: Sexp) -> Vec<Sexp> {
    vec![
        Sexp::Vector(vec![child.clone()]),
        Sexp::Expression {
            elements: vec![attr("first", child.clone())],
            names_present: true,
        },
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![child.clone()]),
            attributes: vec![],
        },
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(vec![child.clone()]),
            attributes: vec![],
        },
        Sexp::AttributedAtomic {
            values: AtomicVector::Integer(vec![Some(1)]),
            attributes: vec![attr("note", child.clone())],
        },
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![]),
            attributes: vec![attr("note", child)],
        },
    ]
}

#[test]
fn typed_and_raw_containers_reject_reachable_embedded_nul_characters() -> Result {
    let owner = Environment::global();
    let mut runtime = RuntimeState::default();
    for value in containers(Sexp::Char(Some("a\0b".into()))) {
        assert!(matches!(
            runtime.eval(&value),
            Err(EvalError::NotImplemented("CHARSXP with embedded NUL"))
        ));
        assert!(matches!(
            ValueObject::import(&owner, &value, ValueImport::Evaluated),
            Err(GraphError::InvalidCharacter)
        ));
    }
    Ok(())
}

#[test]
fn raw_and_managed_typed_children_reject_duplicate_character_metadata() -> Result {
    let owner = Environment::global();
    let invalid = Sexp::AttributedCharacter {
        values: vec![Some("payload".into())],
        attributes: vec![attr("note", string("one")), attr("note", string("two"))],
    };
    let mut runtime = RuntimeState::default();
    for raw in containers(invalid) {
        for value in [raw.clone(), managed(&owner, &raw)?] {
            assert!(matches!(
                runtime.eval(&value),
                Err(EvalError::Type(message)) if message == "duplicate character attribute"
            ));
        }
    }
    Ok(())
}

#[test]
fn valid_character_special_attributes_work_in_raw_typed_and_managed_storage() -> Result {
    let metadata = vec![
        attr("names", string("first")),
        attr("class", string("custom")),
        attr("levels", string("level")),
        attr("row.names", string("row")),
        attr("comment", string("note")),
    ];
    let raw = Sexp::AttributedCharacter {
        values: vec![Some("payload".into())],
        attributes: metadata,
    };
    let typed = Sexp::AttributedAtomic {
        values: AtomicVector::String(vec![Some("payload".into())]),
        attributes: match &raw {
            Sexp::AttributedCharacter { attributes, .. } => attributes.clone(),
            _ => unreachable!(),
        },
    };
    let mut f = Frontend::default();
    let owner = global_environment(&mut f)?;
    for input in [
        raw.clone(),
        typed.clone(),
        managed(&owner, &raw)?,
        managed(&owner, &typed)?,
    ] {
        bind(&mut f, "x", input)?;
        truth(&mut f, "identical(attributes(x),list(names='first',class='custom',levels='level',row.names='row',comment='note'))")?;
    }
    Ok(())
}

#[test]
fn malformed_character_and_typed_metadata_is_not_accepted_as_valid_special_storage() -> Result {
    let malformed = vec![
        vec![ListElement {
            name: None,
            value: string("x"),
        }],
        vec![attr("", string("x"))],
        vec![attr("note", Sexp::Nil)],
        vec![attr("names", Sexp::Integer(vec![Some(1)]))],
        vec![attr("names", Sexp::String(vec![]))],
        vec![attr("class", Sexp::Integer(vec![Some(1)]))],
        vec![attr("class", Sexp::String(vec![]))],
        vec![attr("class", string("factor"))],
        vec![attr("dim", string("one"))],
        vec![attr("dim", Sexp::Integer(vec![Some(2)]))],
        vec![attr("dimnames", Sexp::Vector(vec![string("row")]))],
        vec![attr("row.names", Sexp::Real(vec![1.0]))],
        vec![attr("comment", Sexp::Integer(vec![Some(1)]))],
        vec![attr("tsp", string("bad"))],
    ];
    let mut runtime = RuntimeState::default();
    for attributes in malformed {
        for value in [
            Sexp::AttributedCharacter {
                values: vec![Some("x".into())],
                attributes: attributes.clone(),
            },
            Sexp::AttributedAtomic {
                values: AtomicVector::String(vec![Some("x".into())]),
                attributes,
            },
        ] {
            assert!(runtime.eval(&value).is_err(), "accepted {value:?}");
        }
    }
    Ok(())
}
