mod replace;

use super::*;
use crate::ListElement;

impl ValueObject {
    /// Returns stored attributes with rooted child transports; compact row names stay compact.
    pub fn attribute_values(&self) -> Result<Vec<ListElement>, GraphError> {
        let borrowed = self.borrow()?;
        let Some(description) = borrowed.value().attribute_description()? else {
            return Ok(vec![]);
        };
        Ok(attributes(&description)
            .ok_or(GraphError::WrongKind)?
            .to_vec())
    }

    /// Preserves a no-object alias; otherwise copies only this node and its attribute container.
    pub fn without_class(&self) -> Result<Self, GraphError> {
        let heap = self.0.heap()?;
        let (header, mode, slot) = {
            let store = heap.lock();
            let node = store.node(self.0.id())?;
            let header = snapshot::explicit_header(node);
            if !header.is_object() {
                return Ok(self.clone());
            }
            let Node::Value(value) = node else {
                return Err(GraphError::WrongKind);
            };
            (header, value.mode, attribute_slot(&value.payload))
        };
        let mut filtered = self.attribute_values()?;
        filtered.retain(|attribute| attribute.name.as_deref() != Some("class"));
        if let Some(slot) = slot {
            let prepared = plan::Plan::prepare_attributes(&filtered, heap)?;
            let mut staging = Vec::new();
            let mut store = heap.lock();
            prepared.validate(&store)?;
            let Node::Value(source) = store.node(self.0.id())? else {
                return Err(GraphError::WrongKind);
            };
            let mut copy = source.copied();
            let Payload::Aggregate { slots, .. } = &mut copy.payload else {
                return Err(GraphError::WrongKind);
            };
            let attribute = slots.get_mut(slot).ok_or(GraphError::WrongKind)?;
            copy.header.set_object(false);
            *attribute = prepared.publish(&mut store);
            let id = store.insert(Node::Value(copy));
            return Ok(Self(store.root(heap, id, &mut staging), self.1));
        }
        self.normalized_without_class((header, mode), filtered)
    }

    fn normalized_without_class(
        &self,
        metadata: (SexpHeader, ValueImport),
        filtered: Vec<ListElement>,
    ) -> Result<Self, GraphError> {
        let borrowed = self.borrow()?;
        let mut description = borrowed
            .value()
            .attribute_description()?
            .ok_or(GraphError::WrongKind)?;
        *attributes_mut(&mut description).ok_or(GraphError::WrongKind)? = filtered;
        let heap = self.0.heap()?;
        let mut prepared = plan::Plan::prepare(&description, heap, metadata.1)?;
        let mut header = metadata.0;
        if prepared.sexptype() != header.sexptype() {
            return Err(GraphError::WrongKind);
        }
        header.set_object(false);
        prepared.set_header(header);
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.node(self.0.id())?;
        prepared.validate(&store)?;
        let id = prepared.publish(&mut store);
        Ok(Self(store.root(heap, id, &mut staging), self.1))
    }
}

fn attribute_slot(payload: &Payload) -> Option<usize> {
    match payload {
        Payload::Aggregate {
            shape: Shape::Atomic { .. } | Shape::Character { .. },
            ..
        } => Some(0),
        Payload::Aggregate {
            shape: Shape::Closure { .. },
            ..
        } => Some(3),
        Payload::Aggregate {
            shape: Shape::Aggregate { length, .. },
            ..
        } => Some(*length),
        _ => None,
    }
}

fn attributes(value: &Sexp) -> Option<&[ListElement]> {
    match value {
        Sexp::AttributedAtomic { attributes, .. }
        | Sexp::AttributedAggregate { attributes, .. }
        | Sexp::AttributedCharacter { attributes, .. }
        | Sexp::Closure { attributes, .. } => Some(attributes),
        _ => None,
    }
}

fn attributes_mut(value: &mut Sexp) -> Option<&mut Vec<ListElement>> {
    match value {
        Sexp::AttributedAtomic { attributes, .. }
        | Sexp::AttributedAggregate { attributes, .. }
        | Sexp::AttributedCharacter { attributes, .. }
        | Sexp::Closure { attributes, .. } => Some(attributes),
        _ => None,
    }
}
