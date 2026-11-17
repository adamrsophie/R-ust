use super::*;

#[test]
fn typed_aggregate_extraction_keeps_child_metadata_and_managed_identity() -> Result {
    let mut f = Frontend::default();
    let owner = global_environment(&mut f)?;
    let child = ValueObject::import(
        &owner,
        &Sexp::AttributedCharacter {
            values: vec![Some("a".into()), Some("b".into())],
            attributes: vec![attr("component", Sexp::Integer(vec![Some(13)]))],
        },
        ValueImport::Evaluated,
    )?;
    for expression in [false, true] {
        let raw = Sexp::AttributedAggregate {
            values: if expression {
                AggregateVector::Expression(vec![child.transport()?, Sexp::Nil])
            } else {
                AggregateVector::Vector(vec![child.transport()?, Sexp::Nil])
            },
            attributes: vec![
                attr(
                    "names",
                    Sexp::String(vec![Some("first".into()), Some("second".into())]),
                ),
                attr("note", Sexp::Integer(vec![Some(7)])),
            ],
        };
        for value in [raw.clone(), managed(&owner, &raw)?] {
            bind(&mut f, "x", value)?;
            truth(&mut f, "a<-x[[1L]];b<-x[['first']];identical(a,b) && identical(attr(a,'component'),13L) && is.null(x[['fir']]) && is.null(x[['']]) && is.null(x[[NA_character_]])")?;
            for name in ["a", "b"] {
                let Sexp::Managed(value) = get(&f, name)? else {
                    panic!("selected original child")
                };
                assert!(value.same_object(&child));
            }
            assert!(matches!(
                f.eval_source("x[[3L]]"),
                Err(EvalError::Domain(message)) if message == "subscript out of bounds"
            ));
        }
    }
    Ok(())
}

#[test]
fn typed_expression_extraction_returns_language_without_evaluating_it() -> Result {
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Expression(vec![Sexp::Symbol("never_evaluate".into())]),
        attributes: vec![
            attr("names", string("first")),
            attr("note", Sexp::Integer(vec![Some(7)])),
        ],
    };
    let mut f = Frontend::default();
    let owner = global_environment(&mut f)?;
    for value in [raw.clone(), managed(&owner, &raw)?] {
        bind(&mut f, "x", value)?;
        let values = f.eval_source("x[[1L]];x[['first']]")?;
        assert_eq!(values.len(), 2);
        for value in values {
            assert_eq!(value.snapshot()?, Sexp::Symbol("never_evaluate".into()));
        }
    }
    Ok(())
}

#[test]
fn typed_one_dimensional_aggregate_extraction_uses_effective_dimnames() -> Result {
    let mut f = Frontend::default();
    let owner = global_environment(&mut f)?;
    for labels in [
        Sexp::String(vec![Some("a".into()), Some("b".into())]),
        Sexp::Nil,
    ] {
        let raw = Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![
                Sexp::Integer(vec![Some(7)]),
                Sexp::Integer(vec![Some(8)]),
            ]),
            attributes: vec![
                attr("dim", Sexp::Integer(vec![Some(2)])),
                attr("dimnames", Sexp::Vector(vec![labels.clone()])),
            ],
        };
        for input in [raw.clone(), managed(&owner, &raw)?] {
            bind(&mut f, "x", input)?;
            truth(
                &mut f,
                if matches!(labels, Sexp::Nil) {
                    "is.null(x[['a']]) && identical(x[[1L]],7L)"
                } else {
                    "identical(x[['a']],7L) && identical(x[['b']],8L)"
                },
            )?;
        }
    }
    Ok(())
}
