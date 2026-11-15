use super::heap::{Heap, Node, NodeId, RootPromotion, Store};
use super::values::StoredValue;
use super::{Environment, GraphError, Link};
use crate::{Sexp, SexpHeader};

mod children;
mod duplicate;
mod plan;
mod schema;
mod snapshot;
use schema::Shape;

/// Selects interpretation at an explicit SDK import boundary; neither mode evaluates code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueImport {
    Evaluated,
    Code,
}

/// Typed persistent node schema. This is independent of its public snapshot representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Leaf,
    Explicit,
    Closure,
    Formals,
    Attributes,
    Expression,
    Character,
    Matrix,
    DataFrame,
    Vector,
    List,
    PairList,
    Dots,
    Code,
}

/// A graph-only aggregate handle. It does not activate evaluator normalization.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueObject(Link);

/// Clones extend one semantic lease; a new retain call acquires another owner.
#[derive(Clone, Debug)]
pub struct ValueObjectRetention {
    _root: ValueObject,
}

/// Immediate child handles are GC roots, not additional semantic owners.
#[derive(Clone, Debug)]
pub struct ValueView {
    pub header: SexpHeader,
    pub kind: ValueKind,
    pub mode: Option<ValueImport>,
    pub children: Vec<ValueObject>,
}

pub(super) enum Payload {
    Leaf(StoredValue),
    Aggregate { shape: Shape, slots: Vec<NodeId> },
}

pub(super) struct ValueNode {
    pub header: SexpHeader,
    pub kind: ValueKind,
    pub mode: ValueImport,
    pub payload: Payload,
}

impl ValueNode {
    pub(super) fn edges(&self, output: &mut Vec<NodeId>) {
        match &self.payload {
            Payload::Leaf(value) => value.edges(output),
            Payload::Aggregate { slots, .. } => output.extend(slots),
        }
    }
}

impl ValueObject {
    pub fn import(
        owner: &Environment,
        value: &Sexp,
        mode: ValueImport,
    ) -> Result<Self, GraphError> {
        let heap = owner.0.heap()?;
        let plan = plan::Plan::prepare(value, heap, mode)?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.frame(owner.0.id())?;
        plan.validate(&store)?;
        let id = plan.publish(&mut store);
        Ok(Self(store.root(heap, id, &mut staging)))
    }

    pub fn same_object(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    pub fn semantic_reference_count(&self) -> Result<usize, GraphError> {
        self.0.heap()?.lock().semantic_reference_count(self.0.id())
    }

    pub fn retain(&self) -> Result<ValueObjectRetention, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.node(self.0.id())?;
        let root = store.promote_root(
            self.0.id(),
            &mut RootPromotion::new(heap, &mut staging, true),
        );
        Ok(ValueObjectRetention { _root: Self(root) })
    }

    pub fn view(&self) -> Result<ValueView, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let (header, kind, mode, ids) = match store.node(self.0.id())? {
            Node::Value(node) => (
                node.header,
                node.kind,
                Some(node.mode),
                match &node.payload {
                    Payload::Leaf(_) => Vec::new(),
                    Payload::Aggregate { slots, .. } => slots.clone(),
                },
            ),
            node => (
                snapshot::explicit_header(node),
                ValueKind::Explicit,
                None,
                Vec::new(),
            ),
        };
        let children = ids
            .into_iter()
            .map(|id| Self(store.root(heap, id, &mut staging)))
            .collect();
        Ok(ValueView {
            header,
            kind,
            mode,
            children,
        })
    }

    /// Materializes managed containers; existing explicit graph handles retain identity.
    pub fn snapshot(&self) -> Result<Sexp, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        snapshot::Snapshot {
            heap,
            staging: &mut staging,
            active: Default::default(),
        }
        .read(&mut store, self.0.id())
    }
}
