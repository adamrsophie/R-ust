pub(crate) mod compile;
mod config;
pub(super) use config::enabled as enabled_by_environment;
mod intrinsics;
mod machine;
mod opcode;
mod validate;

use crate::{EvalError, EvalFlow, RuntimeState, Sexp};
use r_ust_sexp::{BytecodeObject, ClosureEnvironment};

impl RuntimeState {
    /// Reports the current execution mode, including host overrides.
    pub fn bytecode_enabled(&self) -> bool {
        self.bytecode_enabled
    }

    pub fn set_bytecode_enabled(&mut self, enabled: bool) {
        self.bytecode_enabled = enabled;
    }

    pub fn compile_closure(&mut self, value: &Sexp) -> Result<Option<Sexp>, EvalError> {
        crate::base::initialize(&self.current)?;
        let Sexp::Closure {
            parameters,
            body,
            environment,
            attributes,
        } = value
        else {
            return Err(EvalError::Type(
                "bytecode compiler requires a closure".into(),
            ));
        };
        if matches!(body.as_ref(), Sexp::Bytecode(_)) {
            return Ok(Some(value.clone()));
        }
        let owner = match environment {
            ClosureEnvironment::Captured(owner) => owner.clone(),
            ClosureEnvironment::Base => self
                .global
                .session_root(r_ust_sexp::EnvironmentRole::BaseNamespace)?,
        };
        Ok(
            compile::expression(&owner, body)?.map(|code| Sexp::Closure {
                parameters: parameters.clone(),
                body: Box::new(Sexp::Bytecode(code)),
                environment: environment.clone(),
                attributes: attributes.clone(),
            }),
        )
    }

    pub(super) fn eval_bytecode(&mut self, code: &BytecodeObject) -> Result<EvalFlow, EvalError> {
        let parts = code.parts()?;
        let version = parts
            .code
            .first()
            .ok_or_else(|| opcode::invalid("missing version"))?;
        if !self.bytecode_enabled || !(9..=12).contains(version) {
            return self.eval_flow(parts.constants.first().unwrap_or(&Sexp::Nil));
        }
        validate::validate(&parts)?;
        machine::execute(self, &parts).map(EvalFlow::Value)
    }
}
