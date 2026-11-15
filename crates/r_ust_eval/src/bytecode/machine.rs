use super::{intrinsics, opcode::*, validate};
use crate::{
    ApplyFunction, EvalError, EvalFlow, EvaluatedArgument, RuntimeState, Sexp, SourceFault,
};
use r_ust_sexp::{BytecodeParts, InternalOperation, PrimitiveId};

struct Frame {
    operation: InternalOperation,
    name: String,
    arguments: Vec<EvaluatedArgument>,
}

pub(super) fn execute(
    runtime: &mut RuntimeState,
    parts: &BytecodeParts,
) -> Result<Sexp, EvalError> {
    let mut values = Vec::new();
    let mut frames: Vec<Frame> = Vec::new();
    let mut pc = 1;
    loop {
        let op = parts.code[pc];
        let mut next = pc + operands(op)? + 1;
        match op {
            0 => {
                return Err(SourceFault::domain(
                    "byte code version mismatch".into(),
                    &runtime.source_current_call(),
                ))
            }
            RETURN => return values.pop().ok_or_else(|| invalid("empty return")),
            15 => runtime.visible = false,
            94 => runtime.visible = true,
            16..=20 => {
                runtime.visible = true;
                let value = match op {
                    LDCONST => validate::constant(parts, parts.code[pc + 1])?.clone(),
                    17 => Sexp::Nil,
                    18 => Sexp::Logical(vec![Some(true)]),
                    19 => Sexp::Logical(vec![Some(false)]),
                    GETVAR => getvar(runtime, validate::symbol(parts, parts.code[pc + 1])?)?,
                    _ => unreachable!(),
                };
                values.push(value);
            }
            GETINTLBUILTIN => {
                let name = validate::symbol(parts, parts.code[pc + 1])?;
                let operation = crate::internal::resolve(name)
                    .ok_or_else(|| invalid("unknown internal builtin"))?;
                frames.push(Frame {
                    operation,
                    name: name.into(),
                    arguments: Vec::new(),
                });
            }
            33..=37 => {
                let value = match op {
                    PUSHARG => values
                        .pop()
                        .ok_or_else(|| invalid("operand stack underflow"))?,
                    34 => validate::constant(parts, parts.code[pc + 1])?.clone(),
                    35 => Sexp::Nil,
                    36 => Sexp::Logical(vec![Some(true)]),
                    37 => Sexp::Logical(vec![Some(false)]),
                    _ => unreachable!(),
                };
                frames
                    .last_mut()
                    .ok_or_else(|| invalid("missing call frame"))?
                    .arguments
                    .push(EvaluatedArgument { name: None, value });
            }
            SETTAG => {
                let name = validate::symbol(parts, parts.code[pc + 1])?;
                let argument = frames
                    .last_mut()
                    .and_then(|frame| frame.arguments.last_mut())
                    .ok_or_else(|| invalid("tag without argument"))?;
                argument.name = Some(name.into());
            }
            CALLBUILTIN => {
                let frame = frames.pop().ok_or_else(|| invalid("missing call frame"))?;
                let call = validate::constant(parts, parts.code[pc + 1])?;
                let flag = intrinsics::print_flag(&frame.name)
                    .ok_or_else(|| invalid("unknown print flag"))?;
                runtime.visible = flag != 1;
                let value = runtime.call_internal(frame.operation, &frame.arguments, Some(call))?;
                if flag < 2 {
                    runtime.visible = flag != 1;
                }
                values.push(value);
            }
            BASEGUARD => {
                let call = validate::constant(parts, parts.code[pc + 1])?;
                let function = runtime.require_source_function(".Internal", true, call)?;
                if function != ApplyFunction::Primitive(PrimitiveId::Internal) {
                    let EvalFlow::Value(value) = runtime.eval_flow(call)?;
                    values.push(value);
                    next = usize::try_from(parts.code[pc + 2])
                        .map_err(|_| invalid("negative jump"))?;
                }
            }
            _ => return Err(EvalError::NotImplemented("bytecode opcode")),
        }
        pc = next;
    }
}

fn getvar(runtime: &mut RuntimeState, name: &str) -> Result<Sexp, EvalError> {
    match runtime.raw_lookup(name)? {
        None => Err(SourceFault::undefined_symbol(
            name,
            &runtime.source_lexical_call(),
        )),
        Some(Sexp::MissingArg(_) | Sexp::MissingValue) => Err(SourceFault::missing(
            name.into(),
            runtime.source_lexical_call(),
            crate::conditions::MissingSite::BytecodeGetvar,
        )),
        Some(value) => runtime.force_value(value),
    }
}
