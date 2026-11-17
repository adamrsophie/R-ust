use super::heap::{Node, NodeId, Store};
use super::values::StoredValue;
use super::{BytecodeObject, Environment, GraphError, Link};
use crate::Sexp;

#[derive(Clone, Debug, PartialEq)]
pub struct BytecodeParts {
    pub code: Vec<i32>,
    pub constants: Vec<Sexp>,
    pub expression: Sexp,
}

pub(super) struct BytecodeNode {
    code: Vec<i32>,
    constants: StoredValue,
    expression: StoredValue,
}

impl BytecodeNode {
    pub(super) fn edges(&self, pending: &mut Vec<NodeId>) {
        self.constants.edges(pending);
        self.expression.edges(pending);
    }
}

impl Store {
    fn bytecode(&self, id: NodeId) -> Result<&BytecodeNode, GraphError> {
        match self.node(id)? {
            Node::Bytecode(value) => Ok(value),
            _ => Err(GraphError::WrongKind),
        }
    }
}

impl BytecodeObject {
    pub fn new(owner: &Environment, parts: BytecodeParts) -> Result<Self, GraphError> {
        let Some(version) = parts.code.first().copied() else {
            return Err(GraphError::InvalidBytecode);
        };
        let mut code = parts.code;
        if !(9..=12).contains(&version) {
            code = vec![version, 0];
        } else {
            code[0] = if code.len() == 2 && code[1] == 0 {
                2
            } else {
                12
            };
        }
        let heap = owner.0.heap()?;
        let constant_values = Sexp::Vector(parts.constants);
        let constants = StoredValue::new(&constant_values, heap)?;
        let expression = StoredValue::new(&parts.expression, heap)?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        store.frame(owner.0.id())?;
        let id = store.insert(Node::Bytecode(Box::new(BytecodeNode {
            code,
            constants,
            expression,
        })));
        Ok(Self(store.root(heap, id, &mut staging)))
    }

    pub fn parts(&self) -> Result<BytecodeParts, GraphError> {
        let heap = self.0.heap()?;
        let mut staging = Vec::new();
        let mut store = heap.lock();
        let node = store.bytecode(self.0.id())?;
        let code = node.code.clone();
        let constants = node.constants.copy_edges();
        let expression = node.expression.copy_edges();
        let Sexp::Vector(constants) = constants.export(&mut store, heap, &mut staging)? else {
            return Err(GraphError::WrongKind);
        };
        let expression = expression.export(&mut store, heap, &mut staging)?;
        Ok(BytecodeParts {
            code,
            constants,
            expression,
        })
    }

    pub fn source(&self) -> Result<Sexp, GraphError> {
        Ok(self
            .parts()?
            .constants
            .into_iter()
            .next()
            .unwrap_or(Sexp::Nil))
    }

    pub fn same_object(&self, other: &Self) -> bool {
        matches!((&self.0, &other.0), (Link::Root(_), Link::Root(_))) && self.0 == other.0
    }

    pub fn display_address(&self) -> Result<String, GraphError> {
        let store = self.0.heap()?.lock();
        Ok(format!("{:p}", store.bytecode(self.0.id())?))
    }
}

pub fn bytecode_source(value: &Sexp) -> Result<Sexp, GraphError> {
    match value {
        Sexp::Bytecode(value) => value.source(),
        Sexp::Managed(value) if value.sexptype() == crate::SexpType::Bytecode => {
            let Sexp::Bytecode(code) = value.transport()? else {
                return Err(GraphError::WrongKind);
            };
            code.source()
        }
        value => Ok(value.clone()),
    }
}
