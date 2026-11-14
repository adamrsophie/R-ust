use super::{control::next_id, ControlTransfer, EvalError, EvalFlow, LoopId, RuntimeState, Sexp};
use r_ust_sexp::{Environment, EvaluationEnvironmentId, EvaluationOrigin};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ContextKind {
    Return(EvaluationEnvironmentId),
    Loop(LoopId),
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Context {
    pub(super) kind: ContextKind,
    pub(super) execution: Environment,
    pub(super) sysparent: Environment,
    pub(super) callable: Sexp,
    pub(super) handler_stack: Option<r_ust_sexp::PairListObject>,
    pub(super) cleanup: std::collections::VecDeque<Sexp>,
    pub(super) arguments: Vec<crate::CallArgument>,
    pub(super) call: Sexp,
    pub(super) generic: bool,
}

pub(super) struct ReturnContext {
    pub(super) execution: Environment,
    pub(super) sysparent: Environment,
    pub(super) callable: Sexp,
    pub(super) arguments: Vec<crate::CallArgument>,
    pub(super) call: Sexp,
    pub(super) generic: bool,
}

impl RuntimeState {
    pub(super) fn with_return_context(
        &mut self,
        entry: ReturnContext,
        evaluate: impl FnOnce(&mut Self) -> Result<EvalFlow, EvalError>,
    ) -> Result<EvalFlow, EvalError> {
        self.with_context(
            Context {
                kind: ContextKind::Return(EvaluationEnvironmentId(next_id()?)),
                execution: entry.execution,
                sysparent: entry.sysparent,
                callable: entry.callable,
                handler_stack: None,
                cleanup: std::collections::VecDeque::new(),
                arguments: entry.arguments,
                call: entry.call,
                generic: entry.generic,
            },
            evaluate,
        )
    }

    pub(super) fn with_context(
        &mut self,
        mut context: Context,
        evaluate: impl FnOnce(&mut Self) -> Result<EvalFlow, EvalError>,
    ) -> Result<EvalFlow, EvalError> {
        context.handler_stack = self.conditions.head.clone();
        let kind = context.kind;
        let saved = std::mem::replace(&mut self.current, context.execution.clone());
        let origin = self.control.origin.clone();
        if let ContextKind::Return(id) = kind {
            self.control.origin = EvaluationOrigin::Frame(id);
        }
        self.control.contexts.push(context);
        let result = evaluate(self);
        let result = self.signal_fresh_result(result);
        let result = self.finish_condition_context(result);
        let result = self.consume_condition_context(result);
        self.current = saved;
        self.control.origin = origin;
        let removed = self.control.contexts.pop();
        debug_assert_eq!(removed.as_ref().map(|entry| entry.kind), Some(kind));
        match (kind, result) {
            (
                ContextKind::Return(id),
                Err(EvalError::Transfer(ControlTransfer::Return(target, value))),
            ) if id == target => Ok(EvalFlow::Value(value)),
            (_, result) => result,
        }
    }

    pub(super) fn eval_selected(
        &mut self,
        entry: ReturnContext,
        expression: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        self.with_return_context(entry, |runtime| runtime.eval_flow(expression))
    }
}

mod select;
#[cfg(test)]
mod tests;
