use std::collections::{HashMap, HashSet};

use super::opcode::*;
use crate::{EvalError, Sexp};
use r_ust_sexp::BytecodeParts;

#[derive(Clone, PartialEq, Eq)]
struct Stack {
    values: usize,
    frames: Vec<usize>,
}

pub(super) fn constant(parts: &BytecodeParts, index: i32) -> Result<&Sexp, EvalError> {
    usize::try_from(index)
        .ok()
        .and_then(|index| parts.constants.get(index))
        .ok_or_else(|| invalid("constant index out of bounds"))
}

pub(super) fn symbol(parts: &BytecodeParts, index: i32) -> Result<&str, EvalError> {
    match constant(parts, index)? {
        Sexp::Symbol(name) => Ok(name),
        _ => Err(invalid("symbol constant required")),
    }
}

pub(super) fn validate(parts: &BytecodeParts) -> Result<(), EvalError> {
    let Some(version) = parts.code.first() else {
        return Err(invalid("missing version"));
    };
    if !(9..=12).contains(version) {
        return Ok(());
    }
    let mut boundaries = HashSet::new();
    let mut pc = 1;
    while pc < parts.code.len() {
        boundaries.insert(pc);
        let count = operands(parts.code[pc])?;
        pc = pc
            .checked_add(count + 1)
            .ok_or_else(|| invalid("instruction overflow"))?;
        if pc > parts.code.len() {
            return Err(invalid("truncated instruction"));
        }
    }
    let initial = Stack {
        values: 0,
        frames: Vec::new(),
    };
    let mut pending = vec![(1usize, initial)];
    let mut seen = HashMap::new();
    while let Some((pc, mut state)) = pending.pop() {
        if !boundaries.contains(&pc) {
            return Err(invalid("fallthrough or invalid jump"));
        }
        if let Some(previous) = seen.insert(pc, state.clone()) {
            if previous != state {
                return Err(invalid("inconsistent stack at jump"));
            }
            continue;
        }
        let op = parts.code[pc];
        let next = pc + operands(op)? + 1;
        match op {
            0 => continue,
            RETURN => {
                if state.values != 1 || !state.frames.is_empty() {
                    return Err(invalid("unbalanced return"));
                }
                continue;
            }
            15 | 94 => {}
            LDCONST => {
                constant(parts, parts.code[pc + 1])?;
                state.values += 1;
            }
            17 | 18 | 19 => state.values += 1,
            GETVAR => {
                symbol(parts, parts.code[pc + 1])?;
                state.values += 1;
            }
            GETINTLBUILTIN => {
                let name = symbol(parts, parts.code[pc + 1])?;
                if crate::internal::resolve(name).is_none()
                    || super::intrinsics::print_flag(name).is_none()
                {
                    return Err(invalid("unknown internal builtin"));
                }
                state.frames.push(0);
            }
            PUSHARG => {
                state.values = state
                    .values
                    .checked_sub(1)
                    .ok_or_else(|| invalid("operand stack underflow"))?;
                *state
                    .frames
                    .last_mut()
                    .ok_or_else(|| invalid("missing call frame"))? += 1;
            }
            34 | 35 | 36 | 37 => {
                if op == 34 {
                    constant(parts, parts.code[pc + 1])?;
                }
                *state
                    .frames
                    .last_mut()
                    .ok_or_else(|| invalid("missing call frame"))? += 1;
            }
            SETTAG => {
                symbol(parts, parts.code[pc + 1])?;
                if state.frames.last().copied().unwrap_or(0) == 0 {
                    return Err(invalid("tag without argument"));
                }
            }
            CALLBUILTIN => {
                if !matches!(constant(parts, parts.code[pc + 1])?, Sexp::Call { .. }) {
                    return Err(invalid("call constant required"));
                }
                state
                    .frames
                    .pop()
                    .ok_or_else(|| invalid("missing call frame"))?;
                state.values += 1;
            }
            BASEGUARD => {
                let Sexp::Call { function, .. } = constant(parts, parts.code[pc + 1])? else {
                    return Err(invalid("guard call required"));
                };
                if !matches!(function.as_ref(), Sexp::Symbol(name) if name == ".Internal") {
                    return Err(invalid("unsupported base guard"));
                }
                let label =
                    usize::try_from(parts.code[pc + 2]).map_err(|_| invalid("negative jump"))?;
                if label <= pc || !boundaries.contains(&label) {
                    return Err(invalid("invalid guard target"));
                }
                let mut skipped = state.clone();
                skipped.values += 1;
                pending.push((label, skipped));
            }
            _ => unreachable!(),
        }
        pending.push((next, state));
    }
    Ok(())
}
