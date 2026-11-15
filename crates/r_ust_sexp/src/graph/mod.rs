mod base_bytecode;
mod bytecode;
mod cons_objects;
pub use bytecode::{bytecode_source, BytecodeParts};
mod environments;
mod frames;
mod heap;
mod list_objects;
#[cfg(test)]
mod pending_assignment_tests;
mod promises;
#[cfg(test)]
mod semantic_ownership_tests;
mod values;

use std::fmt;
use std::sync::Arc;

pub use environments::EnvironmentRole;
use heap::{Heap, NodeId, RootToken};
pub use promises::{Force, ForceLease, PromiseMetadata};

/// An invalid graph access or promise state transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphError {
    Unrooted,
    ForeignHeap,
    StaleNode,
    WrongKind,
    IndexOutOfBounds,
    InvalidCharacter,
    InvalidBytecode,
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

/// A semantic retention lease. Clones extend the same lease rather than creating an alias.
#[derive(Clone, Debug)]
pub struct ValueRetention {
    _value: crate::Sexp,
}

/// A rooted live lexical frame. Clones share identity, not copied bindings.
#[derive(Clone, Debug, PartialEq)]
pub struct Environment(Link);

/// A rooted lazy computation. Metadata and caches are private traced edges.
#[derive(Clone, Debug, PartialEq)]
pub struct PromiseState(Link);

#[derive(Clone, Debug, PartialEq)]
pub struct BytecodeObject(Link);

#[cfg(test)]
mod character_attributes_tests;
#[cfg(test)]
mod closure_attributes_tests;
/// An ordinary traced list whose clones retain shared identity.
#[derive(Clone, Debug, PartialEq)]
pub struct ListObject(Link);

/// An immutable ordinary cons spine whose clones retain head identity.
#[derive(Clone, Debug, PartialEq)]
pub struct PairListObject(Link);

#[cfg(test)]
mod gc_reference_history_tests;

mod value_objects;
pub use value_objects::{ValueImport, ValueKind, ValueObject, ValueObjectRetention, ValueView};
