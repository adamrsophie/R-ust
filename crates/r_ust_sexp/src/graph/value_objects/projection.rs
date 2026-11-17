use super::super::heap::RootToken;
use super::*;
use std::sync::Arc;

/// A read-only typed descriptor. Embedded children are GC-rooted transports, not new owners.
#[derive(Clone, Debug)]
pub struct ValueBorrow {
    owner: ValueObject,
    value: Sexp,
}

impl ValueBorrow {
    pub fn value(&self) -> &Sexp {
        &self.value
    }
    pub fn owner(&self) -> &ValueObject {
        &self.owner
    }
}

impl ValueObject {
    pub fn borrow(&self) -> Result<ValueBorrow, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let value = Projection {
            heap,
            staging: &mut staging,
        }
        .read(&mut store, self.0.id())?;
        Ok(ValueBorrow {
            owner: self.clone(),
            value,
        })
    }
}

pub(super) struct Projection<'a> {
    heap: &'a Arc<Heap>,
    staging: &'a mut Vec<Arc<RootToken>>,
}

impl<'a> Projection<'a> {
    pub(super) fn new(heap: &'a Arc<Heap>, staging: &'a mut Vec<Arc<RootToken>>) -> Self {
        Self { heap, staging }
    }
    pub(super) fn transport(&mut self, store: &mut Store, id: NodeId) -> Result<Sexp, GraphError> {
        match store.node(id)? {
            Node::Value(value) => {
                let sexptype = value.header.sexptype();
                Ok(Sexp::Managed(ValueObject(
                    store.root(self.heap, id, self.staging),
                    sexptype,
                )))
            }
            node => {
                let value = snapshot::explicit_value(node, id)?;
                StoredValue::from_edges(value)?.export(store, self.heap, self.staging)
            }
        }
    }

    fn read(&mut self, store: &mut Store, id: NodeId) -> Result<Sexp, GraphError> {
        let Node::Value(value) = store.node(id)? else {
            return self.transport(store, id);
        };
        match &value.payload {
            Payload::Leaf(value) => {
                let value = value.copy_edges();
                value.export(store, self.heap, self.staging)
            }
            Payload::Aggregate { shape, slots } => {
                let shape = shape.clone();
                let slots = slots.clone();
                let mut values = Vec::with_capacity(slots.len());
                for (index, id) in slots.into_iter().enumerate() {
                    let container = matches!(&shape, Shape::Closure { .. })
                        && (index == 0 || index == 2 || index == 3)
                        || matches!(&shape, Shape::Character { .. } | Shape::Atomic { .. })
                        || matches!(&shape,Shape::Aggregate {length,..} if index==*length);
                    values.push(if container {
                        self.read(store, id)?
                    } else {
                        self.transport(store, id)?
                    });
                }
                shape.restore(values)
            }
        }
    }
}
