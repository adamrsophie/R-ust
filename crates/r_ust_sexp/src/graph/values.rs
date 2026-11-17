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
        match (direct_identity(&self.value), direct_identity(&other.value)) {
            (Some(left), Some(right)) => left == right,
            _ => false,
        }
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

fn direct_identity(value: &Sexp) -> Option<(crate::SexpType, &Link)> {
    let link = match value {
        Sexp::Managed(value) => &value.0,
        Sexp::Bytecode(value) => &value.0,
        Sexp::Environment(value) => &value.0,
        Sexp::ListObject(value) => &value.0,
        Sexp::PairListObject(value) => &value.0,
        Sexp::Promise { state } => &state.0,
        _ => return None,
    };
    Some((value.sexptype(), link))
}
