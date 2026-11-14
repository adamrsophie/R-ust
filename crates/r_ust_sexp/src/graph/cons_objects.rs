use std::sync::Arc;

use super::heap::{Node, NodeId, Store};
use super::values::StoredValue;
use super::{Environment, GraphError, Link, PairListObject};
use crate::ListElement;

pub(super) struct ConsNode {
    pub car: StoredValue,
    pub cdr: Option<NodeId>,
    pub tag: Option<String>,
}

impl Store {
    pub(super) fn cons(&self, id: NodeId) -> Result<&ConsNode, GraphError> {
        match self.node(id)? {
            Node::Cons(value) => Ok(value),
            _ => Err(GraphError::WrongKind),
        }
    }
}

impl PairListObject {
    pub fn cons(
        owner: &Environment,
        car: &ListElement,
        tail: Option<&Self>,
    ) -> Result<Self, GraphError> {
        let heap = owner.0.heap()?;
        let value = StoredValue::new(&car.value, heap)?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.frame(owner.0.id())?;
        let cdr = match tail {
            Some(tail) => {
                if !Arc::ptr_eq(heap, tail.0.heap()?) {
                    return Err(GraphError::ForeignHeap);
                }
                store.cons(tail.0.id())?;
                Some(tail.0.id())
            }
            None => None,
        };
        let id = store.insert(Node::Cons(ConsNode {
            car: value,
            cdr,
            tag: car.name.clone(),
        }));
        Ok(Self(store.root(heap, id, &mut staging)))
    }

    pub fn copy_with(&self, elements: &[ListElement]) -> Result<Option<Self>, GraphError> {
        let heap = self.0.heap()?;
        let values = elements
            .iter()
            .map(|slot| Ok((StoredValue::new(&slot.value, heap)?, slot.name.clone())))
            .collect::<Result<Vec<_>, GraphError>>()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.cons(self.0.id())?;
        let mut tail = None;
        for (car, tag) in values.into_iter().rev() {
            tail = Some(store.insert(Node::Cons(ConsNode {
                car,
                cdr: tail,
                tag,
            })));
        }
        Ok(tail.map(|id| Self(store.root(heap, id, &mut staging))))
    }

    pub fn same_object(&self, other: &Self) -> bool {
        matches!((&self.0, &other.0), (Link::Root(_), Link::Root(_))) && self.0 == other.0
    }

    pub fn car(&self) -> Result<ListElement, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let node = store.cons(self.0.id())?;
        let name = node.tag.clone();
        let value = node.car.copy_edges();
        Ok(ListElement {
            name,
            value: value.export(&mut store, heap, &mut staging)?,
        })
    }

    pub fn cdr(&self) -> Result<Option<Self>, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let tail = store.cons(self.0.id())?.cdr;
        if let Some(id) = tail {
            store.cons(id)?;
        }
        Ok(tail.map(|id| Self(store.root(heap, id, &mut staging))))
    }

    pub fn elements(&self) -> Result<Vec<ListElement>, GraphError> {
        let mut values = Vec::new();
        let mut cursor = Some(self.clone());
        while let Some(head) = cursor {
            values.push(head.car()?);
            cursor = head.cdr()?;
        }
        Ok(values)
    }
}
