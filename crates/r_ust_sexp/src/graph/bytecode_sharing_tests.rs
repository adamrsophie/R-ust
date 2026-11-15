use super::*;
use crate::{
    BytecodeObject, BytecodeParts, ClosureEnvironment, Environment, FunctionParameter, Sexp,
};

#[path = "bytecode_sharing/base.rs"]
mod base;
#[path = "bytecode_sharing/nonfinite.rs"]
mod nonfinite;
#[path = "bytecode_sharing/payload.rs"]
mod payload;

fn code(owner: &Environment, source: Sexp) -> Result<BytecodeObject, GraphError> {
    BytecodeObject::new(
        owner,
        BytecodeParts {
            code: vec![12, 16, 0, 1],
            constants: vec![source],
            expression: Sexp::Nil,
        },
    )
}

fn closure(owner: &Environment, body: Sexp) -> Sexp {
    Sexp::Closure {
        parameters: Vec::new(),
        body: Box::new(body),
        environment: ClosureEnvironment::Captured(owner.clone()),
        attributes: Vec::new(),
    }
}

fn compiled(owner: &Environment, original: &Sexp) -> Result<(Sexp, BytecodeObject), GraphError> {
    let Sexp::Closure { body, .. } = original else {
        return Err(GraphError::WrongKind);
    };
    let code = code(owner, body.as_ref().clone())?;
    let mut replacement = original.clone();
    let Sexp::Closure { body, .. } = &mut replacement else {
        return Err(GraphError::WrongKind);
    };
    *body = Box::new(Sexp::Bytecode(code.clone()));
    Ok((replacement, code))
}

fn references(heap: &Heap, id: NodeId) -> Result<usize, GraphError> {
    heap.lock().semantic_reference_count(id)
}
