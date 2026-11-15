use super::super::heap::RootToken;
use super::*;
use std::collections::HashSet;
use std::sync::Arc;

pub(super) struct Snapshot<'a> {
    pub heap: &'a Arc<Heap>,
    pub staging: &'a mut Vec<Arc<RootToken>>,
    pub active: HashSet<NodeId>,
}

impl Snapshot<'_> {
    pub(super) fn read(&mut self, store: &mut Store, id: NodeId) -> Result<Sexp, GraphError> {
        if !self.active.insert(id) {
            return Err(GraphError::WrongKind);
        }
        let node = match store.node(id)? {
            Node::Value(value) => value,
            node => {
                let value = explicit_value(node, id)?;
                self.active.remove(&id);
                return StoredValue::from_edges(value)?.export(store, self.heap, self.staging);
            }
        };
        let result = match &node.payload {
            Payload::Leaf(value) => {
                let value = value.copy_edges();
                value.export(store, self.heap, self.staging)
            }
            Payload::Aggregate { shape, slots } => {
                let shape = shape.clone();
                let slots = slots.clone();
                let values = slots
                    .into_iter()
                    .map(|id| self.read(store, id))
                    .collect::<Result<Vec<_>, _>>()?;
                shape.restore(values)
            }
        };
        self.active.remove(&id);
        result
    }
}

pub(super) fn explicit_header(node: &Node) -> SexpHeader {
    use crate::SexpType;
    match node {
        Node::Frame(_) => SexpHeader::new(SexpType::Environment),
        Node::Promise(_) => SexpHeader::new(SexpType::Promise),
        Node::Bytecode(_) => SexpHeader::new(SexpType::Bytecode),
        Node::Vector(value) => value.header,
        Node::Cons(_) => SexpHeader::new(SexpType::List),
        Node::Value(value) => value.header,
    }
}

fn explicit_value(node: &Node, id: NodeId) -> Result<Sexp, GraphError> {
    use super::super::{ListObject, PairListObject, PromiseState};
    Ok(match node {
        Node::Frame(_) => Sexp::Environment(Environment(Link::Edge(id))),
        Node::Promise(_) => Sexp::Promise {
            state: PromiseState(Link::Edge(id)),
        },
        Node::Bytecode(_) => Sexp::Bytecode(super::super::BytecodeObject(Link::Edge(id))),
        Node::Vector(_) => Sexp::ListObject(ListObject(Link::Edge(id))),
        Node::Cons(_) => Sexp::PairListObject(PairListObject(Link::Edge(id))),
        Node::Value(_) => return Err(GraphError::WrongKind),
    })
}
