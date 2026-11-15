use r_ust_sexp::{BytecodeObject, BytecodeParts, Environment, PrimitiveId};

use super::opcode::*;
use crate::{CallArgument, EvalError, Sexp};

struct Compiler<'a> {
    owner: &'a Environment,
    words: Vec<i32>,
    constants: Vec<Sexp>,
}

pub(crate) fn expression(
    owner: &Environment,
    source: &Sexp,
) -> Result<Option<BytecodeObject>, EvalError> {
    let mut compiler = Compiler {
        owner,
        words: vec![12],
        constants: vec![Sexp::Nil],
    };
    if !compiler.emit_expression(source)? {
        return Ok(None);
    }
    compiler.constants[0] = r_ust_sexp::canonical_actual_call(source)
        .map_err(|_| EvalError::NotImplemented("bytecode source representation"))?;
    compiler.words.push(RETURN);
    let parts = BytecodeParts {
        code: compiler.words,
        constants: compiler.constants,
        expression: Sexp::Nil,
    };
    super::validate::validate(&parts)?;
    Ok(Some(BytecodeObject::new(owner, parts)?))
}

impl Compiler<'_> {
    fn constant(&mut self, value: Sexp) -> Result<i32, EvalError> {
        let index =
            i32::try_from(self.constants.len()).map_err(|_| invalid("constant table too large"))?;
        self.constants.push(value);
        Ok(index)
    }

    fn emit_expression(&mut self, source: &Sexp) -> Result<bool, EvalError> {
        match source {
            Sexp::Symbol(name)
                if name != "..."
                    && name
                        .strip_prefix("..")
                        .and_then(|name| name.parse::<usize>().ok())
                        .is_none() =>
            {
                let index = self.constant(source.clone())?;
                self.words.extend([GETVAR, index]);
            }
            Sexp::Nil | Sexp::Logical(_) | Sexp::Integer(_) | Sexp::Real(_) | Sexp::String(_) => {
                let index = self.constant(source.clone())?;
                self.words.extend([LDCONST, index]);
            }
            Sexp::Call {
                function,
                arguments,
            } if matches!(function.as_ref(), Sexp::Symbol(name) if name == ".Internal") => {
                return self.emit_internal(source, arguments);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn emit_internal(&mut self, source: &Sexp, outer: &[CallArgument]) -> Result<bool, EvalError> {
        if self.owner.get(".Internal")? != Some(Sexp::Primitive(PrimitiveId::Internal)) {
            return Ok(false);
        }
        let [argument] = outer else { return Ok(false) };
        let Sexp::Call {
            function,
            arguments,
        } = &argument.value
        else {
            return Ok(false);
        };
        let Sexp::Symbol(name) = function.as_ref() else {
            return Ok(false);
        };
        if crate::internal::resolve(name).is_none() || super::intrinsics::print_flag(name).is_none()
        {
            return Ok(false);
        }
        let call_index = self.constant(source.clone())?;
        self.words.extend([BASEGUARD, call_index, 0]);
        let label_slot = self.words.len() - 1;
        let symbol = self.constant(function.as_ref().clone())?;
        self.words.extend([GETINTLBUILTIN, symbol]);
        for argument in arguments {
            if !self.emit_expression(&argument.value)? {
                return Ok(false);
            }
            self.words.push(PUSHARG);
            if let Some(name) = argument.name.as_ref().filter(|name| !name.is_empty()) {
                let index = self.constant(Sexp::Symbol(name.clone()))?;
                self.words.extend([SETTAG, index]);
            }
        }
        let call_index = self.constant(argument.value.clone())?;
        self.words.extend([CALLBUILTIN, call_index]);
        self.words[label_slot] =
            i32::try_from(self.words.len()).map_err(|_| invalid("code too large"))?;
        Ok(true)
    }
}
