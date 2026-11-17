use super::{attribute, check, expected};
use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::{AggregateVector, Environment, ListObject};

#[test]
fn managed_print_mp2_list_labels_encode_missing_literal_and_control_names() -> Result<(), EvalError>
{
    let names = vec![
        Some("a b".into()),
        Some("NA".into()),
        None,
        Some("".into()),
        Some("a\nb".into()),
        Some("a\\b".into()),
        Some("a`b".into()),
    ];
    let values: Vec<_> = (1..=7).map(|n| Sexp::Integer(vec![Some(n)])).collect();
    check(
        "labels",
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(values.clone()),
            attributes: vec![attribute("names", Sexp::String(names.clone()))],
        },
    )?;
    let raw = Sexp::NamedList {
        elements: values
            .into_iter()
            .zip(names)
            .map(|(value, name)| ListElement { name, value })
            .collect(),
        class: None,
        class_first: false,
    };
    let owner = Environment::default();
    let shared = Sexp::ListObject(ListObject::from_value(&owner, &raw)?);
    for value in [&raw, &shared] {
        assert_eq!(super::super::format_sexp(value)?, expected("labels"));
        assert_eq!(
            super::super::character::format(value, true)?,
            expected("labels")
        );
    }
    Ok(())
}

#[test]
fn managed_print_mp2_nested_attribute_labels_use_same_encoding() -> Result<(), EvalError> {
    let named = |name: &str, value: Sexp| Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![value]),
        attributes: vec![attribute("names", Sexp::String(vec![Some(name.into())]))],
    };
    let raw = Sexp::AttributedAggregate {
        values: AggregateVector::Vector(vec![named("a b", Sexp::Integer(vec![Some(1)]))]),
        attributes: vec![
            attribute("names", Sexp::String(vec![Some("outer".into())])),
            attribute(
                "note",
                named("a\nb", named("NA", Sexp::Integer(vec![Some(2)]))),
            ),
        ],
    };
    check("nested_attribute_labels", raw)
}
