mod cons_objects;
mod environments;
mod frames;
mod heap;
mod list_objects;
mod promises;
mod values;

use std::fmt;
use std::sync::Arc;

pub use environments::EnvironmentRole;
use heap::{Heap, NodeId, RootToken};
pub use promises::{Force, ForceLease};

/// An invalid graph access or promise state transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphError {
    Unrooted,
    ForeignHeap,
    StaleNode,
    WrongKind,
    IndexOutOfBounds,
    InvalidCharacter,
    EmptyEnvironment,
    BindingLocked,
    EnvironmentLocked,
    PromiseUnderEvaluation,
    PromiseNotEvaluating,
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "lexical graph error: {self:?}")
    }
}
impl std::error::Error for GraphError {}

#[derive(Clone)]
enum Link {
    Root(Arc<RootToken>),
    Edge(NodeId),
}

impl Link {
    fn id(&self) -> NodeId {
        match self {
            Self::Root(root) => root.id,
            Self::Edge(id) => *id,
        }
    }

    fn heap(&self) -> Result<&Arc<Heap>, GraphError> {
        match self {
            Self::Root(root) => Ok(&root.heap),
            Self::Edge(_) => Err(GraphError::Unrooted),
        }
    }
}

impl PartialEq for Link {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Root(left), Self::Root(right)) => {
                Arc::ptr_eq(&left.heap, &right.heap) && left.id == right.id
            }
            (Self::Edge(left), Self::Edge(right)) => left == right,
            (Self::Root(_), Self::Edge(_)) | (Self::Edge(_), Self::Root(_)) => false,
        }
    }
}

impl fmt::Debug for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Identity").field(&self.id()).finish()
    }
}

/// A rooted live lexical frame. Clones share identity, not copied bindings.
#[derive(Clone, Debug, PartialEq)]
pub struct Environment(Link);

/// A rooted lazy computation. Metadata and caches are private traced edges.
#[derive(Clone, Debug, PartialEq)]
pub struct PromiseState(Link);

#[cfg(test)]
mod character_attributes_tests;
/// An ordinary traced list whose clones retain shared identity.
#[derive(Clone, Debug, PartialEq)]
pub struct ListObject(Link);

/// An immutable ordinary cons spine whose clones retain head identity.
#[derive(Clone, Debug, PartialEq)]
pub struct PairListObject(Link);
