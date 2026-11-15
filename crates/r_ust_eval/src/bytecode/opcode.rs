use crate::EvalError;

pub(super) const RETURN: i32 = 1;
pub(super) const LDCONST: i32 = 16;
pub(super) const GETVAR: i32 = 20;
pub(super) const GETINTLBUILTIN: i32 = 27;
pub(super) const SETTAG: i32 = 31;
pub(super) const PUSHARG: i32 = 33;
pub(super) const CALLBUILTIN: i32 = 39;
pub(super) const BASEGUARD: i32 = 123;

pub(super) fn operands(opcode: i32) -> Result<usize, EvalError> {
    match opcode {
        0 | 1 | 15 | 17 | 18 | 19 | 33 | 35 | 36 | 37 | 94 => Ok(0),
        16 | 20 | 27 | 31 | 34 | 39 => Ok(1),
        123 => Ok(2),
        0..=128 => Err(EvalError::NotImplemented("bytecode opcode")),
        _ => Err(invalid("unknown instruction code")),
    }
}

pub(super) fn invalid(message: &str) -> EvalError {
    EvalError::Type(format!("invalid bytecode: {message}"))
}
