use super::heap::{Frame, Heap, Node, NodeId, Store};
use super::values::StoredValue;
use super::{Environment, GraphError};
use crate::Sexp;

impl Store {
    pub(super) fn frame(&self, id: NodeId) -> Result<&Frame, GraphError> {
        match self.node(id)? {
            Node::Frame(frame) => Ok(frame),
            Node::Promise(_) => Err(GraphError::WrongKind),
        }
    }
    pub(super) fn frame_mut(&mut self, id: NodeId) -> Result<&mut Frame, GraphError> {
        match self.node_mut(id)? {
            Node::Frame(frame) => Ok(frame),
            Node::Promise(_) => Err(GraphError::WrongKind),
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
