use std::sync::atomic::{AtomicU64, Ordering};

use super::dynamic_context::{Context, ContextKind};
use r_ust_sexp::{Environment, EvaluationEnvironmentId, EvaluationOrigin, EvaluationSessionId};

use super::{EvalError, EvalFlow, RuntimeState, Sexp};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoopId(u64);

#[derive(Clone, Debug, PartialEq)]
pub enum ControlTransfer {
    Return(EvaluationEnvironmentId, Sexp),
    Break(LoopId),
    Next(LoopId),
}

impl ControlTransfer {
    pub(super) fn inactive_message(&self) -> &'static str {
        match self {
            Self::Return(..) => "no function to return from, jumping to top level",
            Self::Break(_) | Self::Next(_) => "no loop for break/next, jumping to top level",
        }
    }
}

#[derive(Debug)]
pub(super) struct ControlState {
    pub(super) origin: EvaluationOrigin,
    pub(super) session: EvaluationSessionId,
    pub(super) contexts: Vec<Context>,
}

impl Default for ControlState {
    fn default() -> Self {
        let session = EvaluationSessionId::default();
        Self {
            origin: EvaluationOrigin::Session(session.clone()),
            session,
            contexts: Vec::new(),
        }
    }
}

pub(super) fn next_id() -> Result<u64, EvalError> {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |id| id.checked_add(1))
        .map_err(|_| EvalError::Control("evaluation context identifiers exhausted".into()))
}

impl ControlState {
    fn eligible(&self) -> bool {
        match &self.origin {
            EvaluationOrigin::Global => false,
            EvaluationOrigin::Session(session) => *session == self.session,
            EvaluationOrigin::Frame(id) => self
                .contexts
                .iter()
                .any(|context| context.kind == ContextKind::Return(*id)),
        }
    }

    pub(super) fn return_signal(&self, environment: &Environment, value: Sexp) -> EvalError {
        if self.eligible() {
            for context in self.contexts.iter().rev() {
                if context.execution == *environment {
                    if let ContextKind::Return(id) = context.kind {
                        return EvalError::Transfer(ControlTransfer::Return(id, value));
                    }
                }
            }
        }
        EvalError::Control("no function to return from, jumping to top level".into())
    }

    pub(super) fn loop_signal(
        &self,
        environment: &Environment,
        signal: fn(LoopId) -> ControlTransfer,
    ) -> EvalError {
        if self.eligible() {
            for context in self.contexts.iter().rev() {
                if context.execution == *environment {
                    if let ContextKind::Loop(id) = context.kind {
                        return EvalError::Transfer(signal(id));
                    }
                }
            }
        }
        EvalError::Control("no loop for break/next, jumping to top level".into())
    }
}

pub(super) enum LoopFlow {
    Value(Sexp),
    Break,
    Next,
}

pub(super) fn catch_loop(
    id: LoopId,
    result: Result<EvalFlow, EvalError>,
) -> Result<LoopFlow, EvalError> {
    match result {
        Ok(EvalFlow::Value(value)) => Ok(LoopFlow::Value(value)),
        Err(EvalError::Transfer(ControlTransfer::Break(target))) if target == id => {
            Ok(LoopFlow::Break)
        }
        Err(EvalError::Transfer(ControlTransfer::Next(target))) if target == id => {
            Ok(LoopFlow::Next)
        }
        Err(error) => Err(error),
    }
}

impl RuntimeState {
    pub(super) fn with_loop(
        &mut self,
        evaluate: impl FnOnce(&mut Self, LoopId) -> Result<EvalFlow, EvalError>,
    ) -> Result<EvalFlow, EvalError> {
        let id = LoopId(next_id()?);
        let context = Context {
            kind: ContextKind::Loop(id),
            execution: self.current.clone(),
            sysparent: self
                .global
                .session_root(r_ust_sexp::EnvironmentRole::Base)?,
            callable: Sexp::Nil,
            handler_stack: None,
            cleanup: std::collections::VecDeque::new(),
            arguments: Vec::new(),
            call: Sexp::Nil,
            generic: false,
        };
        self.with_context(context, |runtime| evaluate(runtime, id))
    }
}

#[cfg(test)]
mod tests;
