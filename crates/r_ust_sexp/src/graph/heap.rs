use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, Weak};

use super::environments::{EnvironmentRole, SessionRoots};
use super::promises::Promise;
use super::values::StoredValue;
use super::{GraphError, Link};
#[path = "heap/collection.rs"]
mod collection;
#[path = "heap/ownership.rs"]
mod ownership;
#[path = "heap/promotion.rs"]
mod promotion;
use ownership::semantic_edges;
pub(super) use promotion::RootPromotion;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct NodeId {
    slot: usize,
    generation: u64,
}

impl NodeId {
    pub(super) fn display_token(self) -> String {
        format!("{:016x}{:016x}", self.slot, self.generation)
    }
}

#[derive(Default)]
pub(super) struct Frame {
    pub bindings: HashMap<String, StoredValue>,
    pub assignment_pending: HashSet<String>,
    pub parent: Option<NodeId>,
    pub global: Option<NodeId>,
    pub role: EnvironmentRole,
    pub store: Option<NodeId>,
    pub session: Option<SessionRoots>,
    pub initialized: bool,
    pub bytecode_ready: bool,
}

pub(super) enum Node {
    Frame(Frame),
    Promise(Promise),
    Bytecode(Box<super::bytecode::BytecodeNode>),
    Vector(super::list_objects::VectorNode),
    Cons(super::cons_objects::ConsNode),
}

struct Slot {
    generation: u64,
    node: Option<Node>,
    references: u16,
}

#[derive(Default)]
pub(super) struct Store {
    slots: Vec<Slot>,
    free: Vec<usize>,
    roots: Vec<(NodeId, Weak<RootToken>, bool)>,
}

#[derive(Default)]
pub(super) struct Heap(Mutex<Store>);

pub(super) struct RootToken {
    pub heap: Arc<Heap>,
    pub id: NodeId,
    semantic_owner: bool,
}

impl Drop for RootToken {
    fn drop(&mut self) {
        let mut store = self.heap.lock();
        if self.semantic_owner {
            store.decrement_reference(self.id);
        }
        store.collect();
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
    pub(super) fn insert(&mut self, node: Node) -> NodeId {
        self.acquire_edges(&semantic_edges(&node));
        if let Some(slot) = self.free.pop() {
            self.slots[slot].node = Some(node);
            self.slots[slot].references = 0;
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
                references: 0,
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
}

#[cfg(test)]
#[path = "bytecode_tests.rs"]
mod bytecode_tests;

#[cfg(test)]
#[path = "bytecode_sharing_tests.rs"]
mod bytecode_sharing_tests;

#[path = "session.rs"]
mod session;

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

#[cfg(test)]
#[path = "environment_tests.rs"]
mod environment_tests;

#[cfg(test)]
#[path = "environment_mutation_tests.rs"]
mod environment_mutation_tests;

#[cfg(test)]
#[path = "environment_lifetime_tests.rs"]
mod environment_lifetime_tests;

#[cfg(test)]
#[path = "environment_value_tests.rs"]
mod environment_value_tests;

#[cfg(test)]
#[path = "member_assignment_tests.rs"]
mod member_assignment_tests;

#[cfg(test)]
#[path = "dynamic_context_tests.rs"]
mod dynamic_context_tests;

#[cfg(test)]
#[path = "d1_value_tests.rs"]
mod d1_value_tests;

#[cfg(test)]
#[path = "shared_value_tests.rs"]
mod shared_value_tests;

#[cfg(test)]
#[path = "shared_lifetime_tests.rs"]
mod shared_lifetime_tests;

#[cfg(test)]
#[path = "shared_attributes_tests.rs"]
mod shared_attributes_tests;

#[cfg(test)]
#[path = "import_validation_tests.rs"]
mod import_validation_tests;

#[cfg(test)]
#[path = "expression_tests.rs"]
mod expression_tests;

#[cfg(test)]
#[path = "substitute_quote_tests.rs"]
mod substitute_quote_tests;

#[cfg(test)]
#[path = "list_names_tests.rs"]
mod list_names_tests;
