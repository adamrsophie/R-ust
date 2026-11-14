use super::heap::{Node, NodeId, Store};
use super::values::StoredValue;
use super::{Environment, GraphError, Link, ListObject};
use crate::{ListElement, Sexp, SexpHeader, SexpType};

pub(super) struct VectorNode {
    pub header: SexpHeader,
    pub payload: StoredValue,
}

fn elements(value: &Sexp) -> Result<&[ListElement], GraphError> {
    match value {
        Sexp::List(values)
        | Sexp::ClassedList {
            elements: values, ..
        } => Ok(values),
        _ => Err(GraphError::WrongKind),
    }
}

impl Store {
    pub(super) fn vector(&self, id: NodeId) -> Result<&VectorNode, GraphError> {
        match self.node(id)? {
            Node::Vector(value) => Ok(value),
            _ => Err(GraphError::WrongKind),
        }
    }

    fn vector_mut(&mut self, id: NodeId) -> Result<&mut VectorNode, GraphError> {
        match self.node_mut(id)? {
            Node::Vector(value) => Ok(value),
            _ => Err(GraphError::WrongKind),
        }
    }
}

impl ListObject {
    pub fn from_value(owner: &Environment, value: &Sexp) -> Result<Self, GraphError> {
        elements(value)?;
        let heap = owner.0.heap()?;
        let payload = StoredValue::new(value, heap)?;
        heap.lock().frame(owner.0.id())?;
        Ok(Self(heap.insert(Node::Vector(VectorNode {
            header: SexpHeader::new(SexpType::Vector),
            payload,
        }))))
    }

    pub fn same_object(&self, other: &Self) -> bool {
        matches!((&self.0, &other.0), (Link::Root(_), Link::Root(_))) && self.0 == other.0
    }

    pub fn value(&self) -> Result<Sexp, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let payload = store.vector(self.0.id())?.payload.copy_edges();
        payload.export(&mut store, heap, &mut staging)
    }

    pub fn element(&self, index: usize) -> Result<Sexp, GraphError> {
        let value = self.value()?;
        elements(&value)?
            .get(index)
            .map(|slot| slot.value.clone())
            .ok_or(GraphError::IndexOutOfBounds)
    }

    pub fn header(&self) -> Result<SexpHeader, GraphError> {
        Ok(self.0.heap()?.lock().vector(self.0.id())?.header)
    }

    pub fn set_gp_shared(&self, gp: u16) -> Result<(), GraphError> {
        self.0
            .heap()?
            .lock()
            .vector_mut(self.0.id())?
            .header
            .set_gp(gp);
        Ok(())
    }

    pub fn copy_with(&self, value: &Sexp) -> Result<Self, GraphError> {
        elements(value)?;
        let heap = self.0.heap()?;
        let payload = StoredValue::new(value, heap)?;
        let header = self.header()?;
        Ok(Self(
            heap.insert(Node::Vector(VectorNode { header, payload })),
        ))
    }

    pub fn replace_shared(&self, updates: &[(usize, Sexp)]) -> Result<(), GraphError> {
        let heap = self.0.heap()?;
        let updates = updates
            .iter()
            .map(|(index, value)| Ok((*index, StoredValue::new(value, heap)?)))
            .collect::<Result<Vec<_>, GraphError>>()?;
        let mut store = heap.lock();
        let mut payload = store.vector(self.0.id())?.payload.value.clone();
        if updates.is_empty() {
            return Ok(());
        }
        let slots = match &mut payload {
            Sexp::List(values)
            | Sexp::ClassedList {
                elements: values, ..
            } => values,
            _ => return Err(GraphError::WrongKind),
        };
        for (index, value) in updates {
            let slot = slots.get_mut(index).ok_or(GraphError::IndexOutOfBounds)?;
            slot.value = value.value;
        }
        let payload = StoredValue::from_edges(payload)?;
        store.vector_mut(self.0.id())?.payload = payload;
        store.collect();
        Ok(())
    }
}
