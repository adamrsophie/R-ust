use super::*;
use std::sync::Arc;

enum Prepared {
    Leaf(StoredValue),
    Explicit { id: NodeId, value: StoredValue },
    Aggregate { shape: Shape, children: Vec<Plan> },
}

pub(super) struct Plan {
    header: SexpHeader,
    kind: ValueKind,
    mode: ValueImport,
    payload: Prepared,
}

impl Plan {
    pub(super) fn prepare(
        value: &Sexp,
        heap: &Arc<Heap>,
        mode: ValueImport,
    ) -> Result<Self, GraphError> {
        schema::validate_shape(value)?;
        let header = SexpHeader::new(schema::storage_type(value));
        let kind = schema::kind(value, mode);
        let payload = if kind == ValueKind::Leaf {
            let stored = StoredValue::new(value, heap)?;
            if matches!(
                value,
                Sexp::Bytecode(_)
                    | Sexp::Environment(_)
                    | Sexp::ListObject(_)
                    | Sexp::PairListObject(_)
                    | Sexp::Promise { .. }
            ) {
                let [id] = stored.edges.as_slice() else {
                    return Err(GraphError::WrongKind);
                };
                Prepared::Explicit {
                    id: *id,
                    value: stored,
                }
            } else {
                Prepared::Leaf(stored)
            }
        } else {
            let (shape, sources) = schema::split(value)?;
            let mut children = Vec::with_capacity(sources.len());
            for (source, child_mode, override_kind) in sources {
                let mut child = Self::prepare(&source, heap, child_mode.unwrap_or(mode))?;
                if let Some(kind) = override_kind {
                    child.kind = kind;
                }
                children.push(child);
            }
            Prepared::Aggregate { shape, children }
        };
        Ok(Self {
            header,
            kind,
            mode,
            payload,
        })
    }

    pub(super) fn validate(&self, store: &Store) -> Result<(), GraphError> {
        match &self.payload {
            Prepared::Leaf(value) | Prepared::Explicit { value, .. } => {
                for id in &value.edges {
                    store.node(*id)?;
                }
                match &value.value {
                    Sexp::Environment(value) => {
                        store.frame(value.0.id())?;
                    }
                    Sexp::ListObject(value) => {
                        store.vector(value.0.id())?;
                    }
                    Sexp::PairListObject(value) => {
                        store.cons(value.0.id())?;
                    }
                    Sexp::Bytecode(code) => {
                        if !matches!(store.node(code.0.id())?, Node::Bytecode(_)) {
                            return Err(GraphError::WrongKind);
                        }
                    }
                    Sexp::Promise { state } => {
                        if !matches!(store.node(state.0.id())?, Node::Promise(_)) {
                            return Err(GraphError::WrongKind);
                        }
                    }
                    _ => {}
                }
            }
            Prepared::Aggregate { children, .. } => {
                for child in children {
                    child.validate(store)?;
                }
            }
        }
        Ok(())
    }

    // All fallible checks complete before this transaction publishes its first node.
    pub(super) fn publish(self, store: &mut Store) -> NodeId {
        let payload = match self.payload {
            Prepared::Explicit { id, .. } => return id,
            Prepared::Leaf(value) => Payload::Leaf(value),
            Prepared::Aggregate { shape, children } => Payload::Aggregate {
                shape,
                slots: children
                    .into_iter()
                    .map(|child| child.publish(store))
                    .collect(),
            },
        };
        store.insert(Node::Value(ValueNode {
            header: self.header,
            kind: self.kind,
            mode: self.mode,
            payload,
        }))
    }
}
