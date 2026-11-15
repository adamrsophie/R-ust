use super::heap::{Frame, Heap, Node, NodeId, Store};
use super::values::StoredValue;
use super::{Environment, GraphError};
use crate::Sexp;

impl Store {
    pub(super) fn frame(&self, id: NodeId) -> Result<&Frame, GraphError> {
        match self.node(id)? {
            Node::Frame(frame) => Ok(frame),
            Node::Promise(_)
            | Node::Vector(_)
            | Node::Cons(_)
            | Node::Bytecode(_)
            | Node::Value(_) => Err(GraphError::WrongKind),
        }
    }
    pub(super) fn frame_mut(&mut self, id: NodeId) -> Result<&mut Frame, GraphError> {
        match self.node_mut(id)? {
            Node::Frame(frame) => Ok(frame),
            Node::Promise(_)
            | Node::Vector(_)
            | Node::Cons(_)
            | Node::Bytecode(_)
            | Node::Value(_) => Err(GraphError::WrongKind),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::global()
    }
}

impl Environment {
    pub fn global() -> Self {
        Heap::shared().session()
    }
    pub fn child(&self) -> Result<Self, GraphError> {
        let heap = self.0.heap()?;
        let global = heap
            .lock()
            .frame(self.0.id())?
            .global
            .unwrap_or(self.0.id());
        Ok(Self(heap.insert(Node::Frame(Frame {
            parent: Some(self.0.id()),
            global: Some(global),
            ..Frame::default()
        }))))
    }
    pub fn parent(&self) -> Result<Option<Self>, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let parent = store.frame(self.0.id())?.parent;
        Ok(parent.map(|id| Self(store.root(heap, id, &mut staging))))
    }
    pub fn get_local(&self, name: &str) -> Result<Option<Sexp>, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let value = store
            .binding_frame(self.0.id())?
            .bindings
            .get(name)
            .map(StoredValue::copy_edges);
        value
            .map(|value| value.export(&mut store, heap, &mut staging))
            .transpose()
    }
    pub fn get(&self, name: &str) -> Result<Option<Sexp>, GraphError> {
        let mut current = Some(self.clone());
        while let Some(frame) = current {
            if let Some(value) = frame.get_local(name)? {
                return Ok(Some(value));
            }
            current = frame.parent()?;
        }
        Ok(None)
    }
    pub fn contains_local(&self, name: &str) -> Result<bool, GraphError> {
        Ok(self
            .0
            .heap()?
            .lock()
            .binding_frame(self.0.id())?
            .bindings
            .contains_key(name))
    }
    /// Validates rooted same-heap import without publishing a binding or allocating a node.
    pub fn validate_import(&self, value: &Sexp) -> Result<(), GraphError> {
        let heap = self.0.heap()?;
        heap.lock().frame(self.0.id())?;
        StoredValue::new(value, heap)?;
        Ok(())
    }

    /// Retains one semantic owner independently of evaluator scratch roots.
    pub fn retain_value(&self, value: &Sexp) -> Result<super::ValueRetention, GraphError> {
        let heap = self.0.heap()?;
        let value = StoredValue::new(value, heap)?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.frame(self.0.id())?;
        Ok(super::ValueRetention {
            _value: value.export_promoted(
                &mut store,
                &mut super::heap::RootPromotion::new(heap, &mut staging, true),
            )?,
        })
    }

    /// Marks an existing binding and reports whether a prior assignment is pending.
    pub fn begin_assignment(&self, name: &str) -> Result<Option<bool>, GraphError> {
        let heap = self.0.heap()?;
        let mut store = heap.lock();
        let target = store.writable_frame(self.0.id(), name)?;
        let frame = store.frame_mut(target)?;
        if !frame.bindings.contains_key(name) {
            return Ok(None);
        }
        Ok(Some(!frame.assignment_pending.insert(name.into())))
    }

    /// Clears the binding flag on normal completion, before final root publication.
    pub fn finish_assignment(&self, name: &str) -> Result<bool, GraphError> {
        let heap = self.0.heap()?;
        let mut store = heap.lock();
        let target = store.writable_frame(self.0.id(), name)?;
        Ok(store.frame_mut(target)?.assignment_pending.remove(name))
    }

    pub fn insert(&self, name: String, value: &Sexp) -> Result<(), GraphError> {
        let heap = self.0.heap()?;
        let value = StoredValue::new(value, heap)?;
        let mut store = heap.lock();
        store.write_binding(self.0.id(), name, value)
    }
    pub fn assign_parent(&self, name: String, value: &Sexp) -> Result<(), GraphError> {
        let heap = self.0.heap()?;
        let value = StoredValue::new(value, heap)?;
        let mut store = heap.lock();
        let frame = store.frame(self.0.id())?;
        let mut target = frame.global.unwrap_or(self.0.id());
        let mut current = frame.parent;
        while let Some(id) = current {
            let frame = store.frame(id)?;
            if store.binding_frame(id)?.bindings.contains_key(&name) {
                target = id;
                break;
            }
            current = frame.parent;
        }
        store.write_binding(target, name, value)
    }
}
