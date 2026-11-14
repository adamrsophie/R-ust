use crate::{control::ControlState, dynamic_context::ContextKind, EvalError};
use r_ust_sexp::{Environment, EvaluationEnvironmentId};

impl ControlState {
    pub(crate) fn condition_target(
        &self,
        environment: &Environment,
    ) -> Result<EvaluationEnvironmentId, EvalError> {
        for context in self.contexts.iter().rev() {
            if context.execution == *environment {
                if let ContextKind::Return(target) = context.kind {
                    return Ok(target);
                }
            }
        }
        Err(EvalError::Control(
            "no function to return from, jumping to top level".into(),
        ))
    }
}
