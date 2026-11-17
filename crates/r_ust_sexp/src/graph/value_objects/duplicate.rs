use super::*;

impl ValueNode {
    pub(super) fn copied(&self) -> Self {
        let payload = match &self.payload {
            Payload::Leaf(value) => Payload::Leaf(value.copy_edges()),
            Payload::Aggregate { shape, slots } => Payload::Aggregate {
                shape: shape.clone(),
                slots: slots.clone(),
            },
        };
        Self {
            header: self.header,
            kind: self.kind,
            mode: self.mode,
            payload,
        }
    }
}

impl ValueObject {
    /// Shares formals/body/environment and shallow-copies the attribute container.
    pub fn shallow_copy_closure(&self) -> Result<Self, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let Node::Value(source) = store.node(self.0.id())? else {
            return Err(GraphError::WrongKind);
        };
        let mut copy = source.copied();
        let Payload::Aggregate {
            shape: Shape::Closure { .. },
            slots,
        } = &mut copy.payload
        else {
            return Err(GraphError::WrongKind);
        };
        let [_, _, _, attributes] = slots.as_mut_slice() else {
            return Err(GraphError::WrongKind);
        };
        let Node::Value(attribute_node) = store.node(*attributes)? else {
            return Err(GraphError::WrongKind);
        };
        let attribute_copy = match &attribute_node.payload {
            Payload::Leaf(value) if matches!(value.value, Sexp::Nil) => None,
            Payload::Aggregate { .. } if attribute_node.kind == ValueKind::Attributes => {
                Some(attribute_node.copied())
            }
            _ => return Err(GraphError::WrongKind),
        };
        if let Some(node) = attribute_copy {
            *attributes = store.insert(Node::Value(node));
        }
        let id = store.insert(Node::Value(copy));
        Ok(Self(store.root(heap, id, &mut staging), self.1))
    }
}
