use super::*;

impl ValueObject {
    /// Replaces normalized metadata in a persistent copy, preserving existing payload nodes.
    /// Explicit graph roles and legacy matrices without a lossless attribute slot are rejected.
    pub fn with_attributes(&self, attributes: &[ListElement]) -> Result<Self, GraphError> {
        let heap = self.0.heap()?;
        let (mut header, kind, mode, slot) = {
            let store = heap.lock();
            let Node::Value(value) = store.node(self.0.id())? else {
                return Err(GraphError::WrongKind);
            };
            if value.header.sexptype() != self.1 {
                return Err(GraphError::WrongKind);
            }
            (
                value.header,
                value.kind,
                value.mode,
                attribute_slot(&value.payload),
            )
        };
        let borrowed = self.borrow()?;
        if matches!(borrowed.value(), Sexp::Matrix { .. }) {
            return Err(GraphError::WrongKind);
        }
        let mut description = borrowed
            .value()
            .attribute_description()?
            .ok_or(GraphError::WrongKind)?;
        *attributes_mut(&mut description).ok_or(GraphError::WrongKind)? = attributes.to_vec();
        let replacement = replacement_header(&description)?;
        if replacement.sexptype() != header.sexptype() {
            return Err(GraphError::WrongKind);
        }
        header.set_object(replacement.is_object());
        if let Some(slot) = slot {
            let prepared = plan::Plan::prepare_attributes(attributes, heap)?;
            return self.replace_attribute_slot(prepared, (slot, header));
        }
        let mut prepared = plan::Plan::prepare(&description, heap, mode)?;
        prepared.set_header(header);
        prepared.set_kind(kind);
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.node(self.0.id())?;
        prepared.validate(&store)?;
        let id = prepared.publish(&mut store);
        Ok(Self(store.root(heap, id, &mut staging), self.1))
    }

    fn replace_attribute_slot(
        &self,
        prepared: plan::Plan,
        metadata: (usize, SexpHeader),
    ) -> Result<Self, GraphError> {
        let heap = self.0.heap()?;
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
        let slot = slots.get_mut(metadata.0).ok_or(GraphError::WrongKind)?;
        copy.header = metadata.1;
        *slot = prepared.publish(&mut store);
        let id = store.insert(Node::Value(copy));
        Ok(Self(store.root(heap, id, &mut staging), self.1))
    }
}

fn replacement_header(value: &Sexp) -> Result<SexpHeader, GraphError> {
    schema::validate_shape(value)?;
    let mut header = schema::header(value)?;
    let object = match value {
        Sexp::AttributedCharacter { values, attributes } => {
            schema::validate_attributes(SexpType::String, values.len(), attributes)?
        }
        Sexp::Closure { attributes, .. } => {
            if attributes.iter().any(|attribute| {
                matches!(
                    attribute.name.as_deref(),
                    Some("names" | "dim" | "dimnames")
                )
            }) {
                return Err(GraphError::WrongKind);
            }
            schema::validate_attributes(SexpType::Closure, 1, attributes)?
        }
        _ => header.is_object(),
    };
    header.set_object(object);
    Ok(header)
}
