use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

use super::promises::Promise;
use super::values::StoredValue;
use super::{GraphError, Link};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct NodeId {
    slot: usize,
    generation: u64,
}

pub(super) struct Frame {
    pub bindings: HashMap<String, StoredValue>,
    pub parent: Option<NodeId>,
    pub global: Option<NodeId>,
}

pub(super) enum Node {
    Frame(Frame),
    Promise(Promise),
}

struct Slot {
    generation: u64,
    node: Option<Node>,
}

#[derive(Default)]
pub(super) struct Store {
    slots: Vec<Slot>,
    free: Vec<usize>,
    roots: Vec<(NodeId, Weak<RootToken>)>,
}

#[derive(Default)]
pub(super) struct Heap(Mutex<Store>);

pub(super) struct RootToken {
    pub heap: Arc<Heap>,
    pub id: NodeId,
}

impl Drop for RootToken {
    fn drop(&mut self) {
        self.heap.lock().collect();
    }
}

impl Heap {
    pub fn shared() -> Arc<Self> {
        static LIVE: OnceLock<Mutex<Weak<Heap>>> = OnceLock::new();
        let mut live = LIVE
            .get_or_init(Mutex::default)
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        match live.upgrade() {
            Some(heap) => heap,
            None => {
                let heap = Arc::new(Self::default());
                *live = Arc::downgrade(&heap);
                heap
            }
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, Store> {
        // Every mutation publishes complete edge-only records. No user code runs while locked.
        self.0.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub fn insert(self: &Arc<Self>, node: Node) -> Link {
        let mut staging = Vec::new();
        let mut store = self.lock();
        let id = store.insert(node);
        store.root(self, id, &mut staging)
    }
}

impl Store {
    fn insert(&mut self, node: Node) -> NodeId {
        if let Some(slot) = self.free.pop() {
            self.slots[slot].node = Some(node);
            NodeId {
                slot,
                generation: self.slots[slot].generation,
            }
        } else {
            let id = NodeId {
                slot: self.slots.len(),
                generation: 0,
            };
            self.slots.push(Slot {
                generation: 0,
                node: Some(node),
            });
            id
        }
    }

    pub fn node(&self, id: NodeId) -> Result<&Node, GraphError> {
        self.slots
            .get(id.slot)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.node.as_ref())
            .ok_or(GraphError::StaleNode)
    }

    pub fn node_mut(&mut self, id: NodeId) -> Result<&mut Node, GraphError> {
        self.slots
            .get_mut(id.slot)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.node.as_mut())
            .ok_or(GraphError::StaleNode)
    }

    pub fn root(
        &mut self,
        heap: &Arc<Heap>,
        id: NodeId,
        staging: &mut Vec<Arc<RootToken>>,
    ) -> Link {
        let root = Arc::new(RootToken {
            heap: Arc::clone(heap),
            id,
        });
        // The alias outlives this lock even if a partially promoted value unwinds.
        staging.push(Arc::clone(&root));
        self.roots.push((id, Arc::downgrade(&root)));
        Link::Root(root)
    }

    pub(super) fn collect(&mut self) {
        self.roots.retain(|(_, root)| root.strong_count() != 0);
        let mut pending: Vec<_> = self.roots.iter().map(|(id, _)| *id).collect();
        let mut marked = vec![false; self.slots.len()];
        while let Some(id) = pending.pop() {
            let Ok(node) = self.node(id) else { continue };
            if marked[id.slot] {
                continue;
            }
            marked[id.slot] = true;
            match node {
                Node::Frame(frame) => {
                    pending.extend(frame.parent);
                    pending.extend(frame.global);
                    for value in frame.bindings.values() {
                        value.edges(&mut pending);
                    }
                }
                Node::Promise(promise) => promise.edges(&mut pending),
            }
        }
        for (index, slot) in self.slots.iter_mut().enumerate() {
            if !marked[index] && slot.node.take().is_some() {
                if let Some(next) = slot.generation.checked_add(1) {
                    slot.generation = next;
                    self.free.push(index);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "language_tests.rs"]
mod language_tests;
#[cfg(test)]
#[path = "nested_tests.rs"]
mod nested_tests;
#[cfg(test)]
#[path = "ownership_tests.rs"]
mod ownership_tests;

#[cfg(test)]
#[path = "concurrency_tests.rs"]
mod concurrency_tests;

#[cfg(test)]
#[path = "mutation_tests.rs"]
mod mutation_tests;

#[cfg(test)]
#[path = "primitive_tests.rs"]
mod primitive_tests;
