use super::format_coercion;
use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::{
    AggregateVector, AtomicVector, Environment, EvaluationOrigin, PromiseState, ValueImport,
    ValueObject,
};

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}

fn check(name: &str, raw: Sexp) -> Result<(), EvalError> {
    let expected = include_str!("reference.txt")
        .split(&format!("CASE {name}\n"))
        .nth(1)
        .expect("reference case")
        .split("\nEND\n")
        .next()
        .expect("reference end");
    let owner = Environment::default();
    let object = ValueObject::import(&owner, &raw, ValueImport::Evaluated)?;
    let before = object.view()?;
    let references = object.semantic_reference_count()?;
    let descriptor = object.borrow()?;
    let transport = object.transport()?;
    drop(owner);
    for value in [&raw, &transport, descriptor.value()] {
        assert_eq!(format_coercion(value)?, expected, "{name}");
    }
    let after = object.view()?;
    assert_eq!(before.header, after.header);
    assert_eq!(before.children.len(), after.children.len());
    for (a, b) in before.children.iter().zip(&after.children) {
        assert!(a.same_object(b));
    }
    assert_eq!(object.semantic_reference_count()?, references);
    Ok(())
}

#[test]
fn coercion_deparse_atomic_names_retained_other_attributes_suppressed() -> Result<(), EvalError> {
    let names = || {
        attribute(
            "names",
            Sexp::String(vec![Some("a".into()), Some("b".into())]),
        )
    };
    let note = || attribute("note", Sexp::Integer(vec![Some(7)]));
    for (name, values, attributes) in [
        ("attributes", vec![Some(1), Some(2), Some(3)], vec![note()]),
        (
            "named_integer",
            vec![Some(1), Some(4)],
            vec![names(), note()],
        ),
        (
            "array",
            vec![Some(1), Some(4), Some(9), Some(16)],
            vec![
                attribute("dim", Sexp::Integer(vec![Some(2), Some(2)])),
                attribute(
                    "dimnames",
                    Sexp::AttributedAggregate {
                        values: AggregateVector::Vector(vec![
                            Sexp::String(vec![Some("a".into()), Some("b".into())]),
                            Sexp::String(vec![Some("x".into()), Some("y".into())]),
                        ]),
                        attributes: vec![],
                    },
                ),
            ],
        ),
        (
            "scalar",
            vec![Some(7)],
            vec![
                attribute("names", Sexp::String(vec![Some("x".into())])),
                attribute("class", Sexp::String(vec![Some("custom".into())])),
            ],
        ),
    ] {
        check(
            name,
            Sexp::AttributedAtomic {
                values: AtomicVector::Integer(values),
                attributes,
            },
        )?;
    }
    Ok(())
}

#[test]
fn coercion_deparse_logical_character_and_expression_values() -> Result<(), EvalError> {
    check(
        "logical",
        Sexp::AttributedAtomic {
            values: AtomicVector::Logical(vec![Some(true), None, Some(false)]),
            attributes: vec![attribute(
                "names",
                Sexp::String(vec![Some("a".into()), Some("b".into()), Some("c".into())]),
            )],
        },
    )?;
    check(
        "character",
        Sexp::AttributedAtomic {
            values: AtomicVector::String(vec![Some("a".into()), Some("b".into())]),
            attributes: vec![attribute(
                "names",
                Sexp::String(vec![Some("x".into()), Some("y".into())]),
            )],
        },
    )?;
    check(
        "expression",
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(vec![
                Sexp::Symbol("x".into()),
                Sexp::Integer(vec![Some(1)]),
            ]),
            attributes: vec![
                attribute(
                    "names",
                    Sexp::String(vec![Some("a".into()), Some("b".into())]),
                ),
                attribute("note", Sexp::Integer(vec![Some(7)])),
            ],
        },
    )
}

#[test]
fn coercion_deparse_nested_list_attributes_are_suppressed() -> Result<(), EvalError> {
    check(
        "list",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(vec![
                Sexp::AttributedAtomic {
                    values: AtomicVector::Integer(vec![Some(1), Some(4)]),
                    attributes: vec![attribute("note", Sexp::Integer(vec![Some(7)]))],
                },
                Sexp::Logical(vec![Some(true)]),
            ]),
            attributes: vec![
                attribute(
                    "names",
                    Sexp::String(vec![Some("x".into()), Some("y".into())]),
                ),
                attribute("note", Sexp::Integer(vec![Some(8)])),
            ],
        },
    )
}

#[test]
fn coercion_deparse_backticks_cutoff_and_newlines_match_gnu() -> Result<(), EvalError> {
    check("symbol", Sexp::Symbol("not syntactic".into()))?;
    check(
        "multiline",
        Sexp::String(vec![Some("abcdefghij".into()); 50]),
    )?;
    check("nil", Sexp::Nil)
}

#[test]
fn coercion_deparse_promise_requires_explicit_runtime_without_forcing() -> Result<(), EvalError> {
    let owner = Environment::default();
    let expression = Sexp::Symbol("UNFORCED".into());
    let state = PromiseState::new(&expression, &owner, EvaluationOrigin::Global, true)?;
    let value = Sexp::Promise {
        state: state.clone(),
    };
    assert_eq!(
        format_coercion(&value),
        Err(EvalError::NotImplemented("language printer node"))
    );
    assert_eq!(state.expression()?, expression);
    assert!(state.environment()?.is_some());
    Ok(())
}

mod edges;
