use super::super::value_objects::ValueObject;
use super::*;
use crate::{
    AggregateVector, AtomicVector, ClosureEnvironment, Environment, FunctionParameter, ListElement,
    Sexp, SexpType, ValueImport, ValueKind,
};

mod lifetime;
mod normalized;
mod storage;
mod validation;

type Result = std::result::Result<(), GraphError>;

fn integer(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}
fn text(value: &str) -> Sexp {
    Sexp::String(vec![Some(value.into())])
}
fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}
fn vector(values: Vec<Sexp>, attributes: Vec<ListElement>) -> Sexp {
    Sexp::AttributedAggregate {
        values: AggregateVector::Vector(values),
        attributes,
    }
}
fn state(heap: &Heap) -> Vec<(u64, bool, u16)> {
    heap.lock()
        .slots
        .iter()
        .map(|slot| (slot.generation, slot.node.is_some(), slot.references))
        .collect()
}
fn live(heap: &Heap) -> usize {
    heap.lock()
        .slots
        .iter()
        .filter(|slot| slot.node.is_some())
        .count()
}
fn set_gp(value: &ValueObject, gp: u16) -> Result {
    let heap = value.0.heap()?;
    let mut store = heap.lock();
    let Node::Value(node) = store.node_mut(value.0.id())? else {
        return Err(GraphError::WrongKind);
    };
    node.header.set_gp(gp);
    Ok(())
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
