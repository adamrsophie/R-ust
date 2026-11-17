use super::*;
use crate::{
    AggregateVector, AtomicVector, Environment, GraphError, ListElement, Sexp, SexpType,
    ValueImport, ValueKind, ValueObject,
};

mod lifetime;
mod metadata;
mod specialized;
mod storage;
mod validation;

type Result = std::result::Result<(), GraphError>;

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
fn text(value: &str) -> Sexp {
    Sexp::String(vec![Some(value.into())])
}
fn integer(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}
fn vector(values: Vec<Sexp>, attributes: Vec<ListElement>) -> Sexp {
    Sexp::AttributedAggregate {
        values: AggregateVector::Vector(values),
        attributes,
    }
}
fn description(value: &Sexp) -> std::result::Result<Sexp, GraphError> {
    value.attribute_description()?.ok_or(GraphError::WrongKind)
}
fn snapshots(attributes: &[ListElement]) -> std::result::Result<Vec<ListElement>, GraphError> {
    attributes
        .iter()
        .map(|a| {
            Ok(ListElement {
                name: a.name.clone(),
                value: a.value.snapshot()?,
            })
        })
        .collect()
}
