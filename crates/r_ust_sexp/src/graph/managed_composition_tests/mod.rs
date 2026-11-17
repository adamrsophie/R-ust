use super::*;
use crate::{
    AggregateVector, AtomicVector, BytecodeObject, BytecodeParts, ClosureEnvironment, Environment,
    FunctionParameter, ListElement, Sexp, SexpType, ValueImport, ValueObject,
};

mod bytecode;
mod nonfinite;

type Result = std::result::Result<(), GraphError>;

fn integer(value: i32) -> Sexp {
    Sexp::Integer(vec![Some(value)])
}

fn attribute(name: &str, value: Sexp) -> ListElement {
    ListElement {
        name: Some(name.into()),
        value,
    }
}

fn code(owner: &Environment, source: Sexp) -> std::result::Result<BytecodeObject, GraphError> {
    BytecodeObject::new(
        owner,
        BytecodeParts {
            code: vec![12, 16, 0, 1],
            constants: vec![source],
            expression: Sexp::Nil,
        },
    )
}

fn live_nodes(heap: &Heap) -> usize {
    heap.lock()
        .slots
        .iter()
        .filter(|slot| slot.node.is_some())
        .count()
}
