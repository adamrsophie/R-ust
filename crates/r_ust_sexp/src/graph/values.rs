use std::sync::Arc;

use super::heap::{Heap, NodeId, RootPromotion, RootToken, Store};
use super::{GraphError, Link};
use crate::Sexp;
#[path = "values/visit.rs"]
mod walk;
use walk::visit;

pub(super) struct StoredValue {
    pub(super) value: Sexp,
    pub(super) edges: Vec<NodeId>,
}

impl StoredValue {
    pub fn new(value: &Sexp, heap: &Arc<Heap>) -> Result<Self, GraphError> {
        let mut value = value.clone();
        let mut edges = Vec::new();
        visit(&mut value, &mut |link| {
            if !Arc::ptr_eq(link.heap()?, heap) {
                return Err(GraphError::ForeignHeap);
            }
            let id = link.id();
            edges.push(id);
            *link = Link::Edge(id);
            Ok(())
        })?;
        Ok(Self { value, edges })
    }

    pub(super) fn from_edges(mut value: Sexp) -> Result<Self, GraphError> {
        let mut edges = Vec::new();
        visit(&mut value, &mut |link| {
            match link {
                Link::Edge(id) => edges.push(*id),
                Link::Root(_) => return Err(GraphError::WrongKind),
            }
            Ok(())
        })?;
        Ok(Self { value, edges })
    }

    pub fn edges(&self, pending: &mut Vec<NodeId>) {
        pending.extend(&self.edges);
    }

    pub fn export(
        &self,
        store: &mut Store,
        heap: &Arc<Heap>,
        staging: &mut Vec<Arc<RootToken>>,
    ) -> Result<Sexp, GraphError> {
        self.export_promoted(store, &mut RootPromotion::new(heap, staging, false))
    }

    pub(super) fn export_promoted(
        &self,
        store: &mut Store,
        promotion: &mut RootPromotion<'_>,
    ) -> Result<Sexp, GraphError> {
        let mut value = self.value.clone();
        visit(&mut value, &mut |link| {
            let id = link.id();
            store.node(id)?;
            *link = store.promote_root(id, promotion);
            Ok(())
        })?;
        Ok(value)
    }

    pub(super) fn same_direct_reference(&self, other: &Self) -> bool {
        matches!((&self.value, &other.value),
            (Sexp::ListObject(left), Sexp::ListObject(right)) if left == right)
            || matches!((&self.value, &other.value),
                (Sexp::PairListObject(left), Sexp::PairListObject(right)) if left == right)
            || matches!((&self.value, &other.value),
                (Sexp::Environment(left), Sexp::Environment(right)) if left == right)
            || matches!((&self.value, &other.value),
                (Sexp::Bytecode(left), Sexp::Bytecode(right)) if left == right)
            || matches!((&self.value, &other.value),
                (Sexp::Promise { state: left }, Sexp::Promise { state: right }) if left == right)
    }

    pub fn copy_edges(&self) -> Self {
        Self {
            value: self.value.clone(),
            edges: self.edges.clone(),
        }
    }
}

pub(super) fn same_snapshot(left: &Sexp, right: &Sexp) -> Result<bool, GraphError> {
    let mut left = left.clone();
    let mut right = right.clone();
    let left_bits = walk::real_bits(&mut left)?;
    let right_bits = walk::real_bits(&mut right)?;
    Ok(left == right && left_bits == right_bits)
}
