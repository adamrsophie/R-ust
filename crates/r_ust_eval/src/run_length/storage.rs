use crate::{EvalError, ListElement, Sexp};
use r_ust_sexp::AggregateVector;

pub(super) fn read(value: &Sexp) -> Result<Sexp, EvalError> {
    if let Sexp::Managed(value) = value {
        let borrowed = value.borrow()?;
        return Ok(crate::ordinary_values::read(borrowed.value())?.into_owned());
    }
    Ok(crate::ordinary_values::read(value)?.into_owned())
}

pub(super) fn attributes(value: &Sexp) -> Result<Vec<ListElement>, EvalError> {
    let value = read(value)?;
    if let Sexp::PairList(elements) = &value {
        return Ok(if elements.as_slice().iter().any(|e| e.name.is_some()) {
            vec![ListElement {
                name: Some("names".into()),
                value: Sexp::String(elements.as_slice().iter().map(|e| e.name.clone()).collect()),
            }]
        } else {
            vec![]
        });
    }
    Ok(match value.attribute_description()? {
        Some(Sexp::AttributedAtomic { attributes, .. })
        | Some(Sexp::AttributedAggregate { attributes, .. })
        | Some(Sexp::AttributedCharacter { attributes, .. })
        | Some(Sexp::Closure { attributes, .. }) => attributes,
        _ => vec![],
    })
}

pub(super) fn payload(value: &Sexp) -> Result<Sexp, EvalError> {
    let value = read(value)?;
    Ok(match value.attribute_description()?.unwrap_or(value) {
        Sexp::AttributedAtomic { values, .. } => values.into_value(),
        Sexp::AttributedCharacter { values, .. } => Sexp::String(values),
        Sexp::AttributedAggregate {
            values: AggregateVector::Vector(values),
            ..
        } => Sexp::Vector(values),
        Sexp::AttributedAggregate {
            values: AggregateVector::Expression(values),
            ..
        } => Sexp::Expression {
            elements: values
                .into_iter()
                .map(|value| ListElement { name: None, value })
                .collect(),
            names_present: false,
        },
        other => other,
    })
}
