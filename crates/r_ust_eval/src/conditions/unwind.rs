use crate::{
    dynamic_context::ContextKind, ControlTransfer, EvalError, EvalFlow, RuntimeState, Sexp,
};

impl super::ConditionJump {
    pub(crate) fn fixup(&self) -> Result<(), EvalError> {
        if let (Some(message), Some(expected), Sexp::Nil, Sexp::ListObject(token)) = (
            &self.message,
            &self.token,
            self.result.element(0)?,
            self.result.element(3)?,
        ) {
            if token.same_object(expected) {
                self.result
                    .replace_shared(&[(0, Sexp::String(vec![Some(message.clone())]))])?;
            }
        }
        Ok(())
    }
}

impl RuntimeState {
    pub(crate) fn consume_condition_context(
        &self,
        pending: Result<EvalFlow, EvalError>,
    ) -> Result<EvalFlow, EvalError> {
        let Err(EvalError::Condition(jump)) = pending else {
            return pending;
        };
        let Some(context) = self.control.contexts.last() else {
            return Err(EvalError::Condition(jump));
        };
        if context.kind != crate::dynamic_context::ContextKind::Return(jump.target) {
            return Err(EvalError::Condition(jump));
        }
        if self.control.condition_target(&context.execution)? != jump.target {
            return Err(EvalError::Control("inactive condition target".into()));
        }
        Ok(EvalFlow::Value(Sexp::ListObject(jump.result)))
    }

    pub(crate) fn finish_condition_context(
        &mut self,
        mut pending: Result<EvalFlow, EvalError>,
    ) -> Result<EvalFlow, EvalError> {
        let Some(context) = self.control.contexts.last() else {
            return Err(EvalError::Control("missing exiting context".into()));
        };
        let saved_head = context.handler_stack.clone();
        if pending.is_err() {
            let target = jump_target(&pending);
            if context.cleanup.is_empty() && target != Some(context.kind) {
                return pending;
            }
            self.conditions.head = saved_head.clone();
        } else {
            self.conditions.unwind(&saved_head)?;
        }
        let mut saved_visibility = self.visible;
        loop {
            let action = self
                .control
                .contexts
                .last_mut()
                .and_then(|context| context.cleanup.pop_front());
            let Some(action) = action else {
                break;
            };
            if let Err(EvalError::Condition(jump)) = &pending {
                jump.fixup()?;
            }
            let Some(context) = self.control.contexts.last() else {
                return Err(EvalError::Control("missing cleanup context".into()));
            };
            self.current = context.execution.clone();
            let remaining = context.cleanup.clone();
            match self.eval_flow(&action) {
                Ok(_) => {
                    let Some(context) = self.control.contexts.last_mut() else {
                        return Err(EvalError::Control("missing cleanup context".into()));
                    };
                    context.cleanup = remaining;
                }
                Err(error) => {
                    pending = Err(error);
                    saved_visibility = self.visible;
                    self.conditions.head = saved_head.clone();
                }
            }
        }
        self.visible = saved_visibility;
        pending
    }
}

fn jump_target(pending: &Result<EvalFlow, EvalError>) -> Option<ContextKind> {
    match pending {
        Err(EvalError::Condition(jump)) => Some(ContextKind::Return(jump.target)),
        Err(EvalError::Transfer(ControlTransfer::Return(target, _))) => {
            Some(ContextKind::Return(*target))
        }
        Err(EvalError::Transfer(
            ControlTransfer::Break(target) | ControlTransfer::Next(target),
        )) => Some(ContextKind::Loop(*target)),
        _ => None,
    }
}
