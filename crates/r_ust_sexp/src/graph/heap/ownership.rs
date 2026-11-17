use super::*;

impl Store {
    pub(in crate::graph) fn semantic_reference_count(
        &self,
        target: NodeId,
    ) -> Result<usize, GraphError> {
        self.node(target)?;
        Ok(usize::from(self.slots[target.slot].references))
    }

    #[cfg(test)]
    pub(in crate::graph) fn audit_reference_count(
        &self,
        target: NodeId,
    ) -> Result<usize, GraphError> {
        self.node(target)?;
        let roots = self
            .roots
            .iter()
            .filter(|(id, root, semantic)| *id == target && *semantic && root.strong_count() != 0)
            .count();
        Ok(self
            .slots
            .iter()
            .filter_map(|slot| slot.node.as_ref())
            .map(|node| {
                semantic_edges(node)
                    .iter()
                    .filter(|id| **id == target)
                    .count()
            })
            .fold(roots, usize::saturating_add))
    }

    pub(super) fn increment_reference(&mut self, id: NodeId) {
        if let Some(slot) = self
            .slots
            .get_mut(id.slot)
            .filter(|slot| slot.generation == id.generation && slot.node.is_some())
        {
            slot.references = slot.references.saturating_add(1);
        }
    }

    pub(super) fn decrement_reference(&mut self, id: NodeId) {
        if let Some(slot) = self
            .slots
            .get_mut(id.slot)
            .filter(|slot| slot.generation == id.generation && slot.node.is_some())
        {
            if slot.references != u16::MAX {
                slot.references = slot.references.saturating_sub(1);
            }
        }
    }

    pub(in crate::graph) fn acquire_edges(&mut self, edges: &[NodeId]) {
        for id in edges {
            self.increment_reference(*id);
        }
    }

    pub(in crate::graph) fn release_edges(&mut self, edges: &[NodeId]) {
        for id in edges {
            self.decrement_reference(*id);
        }
    }

    // Direct graph handles follow one R-style slot update. Inline SDK snapshots
    // release and acquire their described edges; they do not have R object identity.
    pub(in crate::graph) fn replace_value_references(
        &mut self,
        old: &StoredValue,
        new: &StoredValue,
    ) {
        if old.same_direct_reference(new) {
            return;
        }
        self.release_edges(&old.edges);
        self.acquire_edges(&new.edges);
    }
}

pub(super) fn semantic_edges(node: &Node) -> Vec<NodeId> {
    let mut edges = Vec::new();
    match node {
        Node::Frame(frame) => {
            if let Some(attributes) = &frame.attributes {
                attributes.edges(&mut edges);
            }
            for value in frame.bindings.values() {
                value.edges(&mut edges);
            }
        }
        Node::Promise(promise) => promise.semantic_edges(&mut edges),
        Node::Bytecode(code) => code.edges(&mut edges),
        Node::Value(value) => value.edges(&mut edges),
        Node::Vector(vector) => vector.payload.edges(&mut edges),
        Node::Cons(cons) => {
            cons.car.edges(&mut edges);
            edges.extend(cons.cdr);
        }
    }
    edges
}
