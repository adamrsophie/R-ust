use std::sync::atomic::{AtomicU64, Ordering};

use r_ust_sexp::{EvaluationEnvironmentId, EvaluationOrigin};

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

#[derive(Debug, Default)]
pub(super) struct ControlState {
    pub(super) origin: EvaluationOrigin,
    returns: Vec<EvaluationEnvironmentId>,
    loops: Vec<(LoopId, EvaluationOrigin)>,
}

fn next_id() -> Result<u64, EvalError> {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |id| id.checked_add(1))
        .map_err(|_| EvalError::Control("evaluation context identifiers exhausted".into()))
}

impl ControlState {
    pub(super) fn enter_return_context(
        &mut self,
    ) -> Result<(EvaluationEnvironmentId, EvaluationOrigin), EvalError> {
        let id = EvaluationEnvironmentId(next_id()?);
        let previous = std::mem::replace(&mut self.origin, EvaluationOrigin::Frame(id));
        self.returns.push(id);
        Ok((id, previous))
    }

    pub(super) fn leave_return_context(
        &mut self,
        id: EvaluationEnvironmentId,
        previous: EvaluationOrigin,
        result: Result<EvalFlow, EvalError>,
    ) -> Result<EvalFlow, EvalError> {
        self.origin = previous;
        let removed = self.returns.pop();
        debug_assert_eq!(removed, Some(id));
        match result {
            Err(EvalError::Transfer(ControlTransfer::Return(target, value))) if target == id => {
                Ok(EvalFlow::Value(value))
            }
            other => other,
        }
    }

    pub(super) fn return_signal(&self, value: Sexp) -> EvalError {
        if let EvaluationOrigin::Frame(id) = self.origin {
            if self.returns.contains(&id) {
                return EvalError::Transfer(ControlTransfer::Return(id, value));
            }
        }
        EvalError::Control("no function to return from, jumping to top level".into())
    }

    pub(super) fn loop_signal(&self, signal: fn(LoopId) -> ControlTransfer) -> EvalError {
        match self
            .loops
            .iter()
            .rev()
            .find(|(_, origin)| *origin == self.origin)
        {
            Some((id, _)) => EvalError::Transfer(signal(*id)),
            None => EvalError::Control("no loop for break/next, jumping to top level".into()),
        }
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
        self.control.loops.push((id, self.control.origin));
        let result = evaluate(self, id);
        let removed = self.control.loops.pop();
        debug_assert_eq!(removed, Some((id, self.control.origin)));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Frontend;

    #[test]
    fn control_and_scope_stacks_restore_after_all_exits() {
        for (index, source) in [
            "(function(){f<-function(x)x;f(return(42L));99L})()",
            "f<-function(x)x;repeat{f(break)}",
            "f<-function(x)x;for(i in 1:2)f(next)",
            "f<-function(x)x;while(TRUE)f(break)",
            "f<-function(x){private<-1L;x};f(missing_name)",
            "with(list(private=1L),missing_name)",
            "(function(){while(TRUE)missing_name})()",
            "(function(){repeat{quit(status=0)}})()",
        ]
        .into_iter()
        .enumerate()
        {
            let mut frontend = Frontend::default();
            let result = frontend.eval_source(source);
            assert_eq!(result.is_ok(), index < 4, "{source}: {result:?}");
            assert!(
                matches!(
                    &result,
                    Ok(_) | Err(EvalError::UndefinedSymbol(_) | EvalError::Quit(_))
                ),
                "{source}: {result:?}"
            );
            let runtime = frontend.runtime();
            assert_eq!(runtime.scopes.len(), 1, "{source}");
            assert_eq!(runtime.control.origin, EvaluationOrigin::Global, "{source}");
            assert!(runtime.control.returns.is_empty(), "{source}");
            assert!(runtime.control.loops.is_empty(), "{source}");
            assert!(runtime.get("private").is_none(), "{source}");
            assert_eq!(
                frontend.eval_source("42L").unwrap(),
                [Sexp::Integer(vec![Some(42)])]
            );
        }
    }

    #[test]
    fn data_masks_own_local_control_and_forward_promises() {
        for (source, expected) in [
            ("(function(){with(list(a=1L),return(3L));99L})()", 99),
            (
                "(function(){f<-function(x)with(list(a=1L),x);f(return(3L));99L})()",
                3,
            ),
            (
                "f<-function(x)with(list(a=1L),x);for(i in 1:3){f(break);cat('NO')};42L",
                42,
            ),
            (
                "(function(){subset(data.frame(a=1:3),return(TRUE));99L})()",
                99,
            ),
            (
                "(function(){f<-function(x)subset(data.frame(a=1:3),x);f(return(3L));99L})()",
                3,
            ),
        ] {
            let mut frontend = Frontend::default();
            assert_eq!(
                frontend.eval_source(source).unwrap().last(),
                Some(&Sexp::Integer(vec![Some(expected)])),
                "{source}"
            );
            assert!(frontend.runtime_mut().take_output().is_empty(), "{source}");
            assert_eq!(frontend.runtime().scopes.len(), 1);
            assert!(frontend.runtime().control.returns.is_empty());
            assert!(frontend.runtime().control.loops.is_empty());
        }
        for signal in ["break", "next"] {
            let mut frontend = Frontend::default();
            let source = format!(
                "(function(){{for(i in 1:3){{with(list(a=1L),{signal});cat('NO')}};42L}})()"
            );
            assert_eq!(
                frontend.eval_source(&source),
                Err(EvalError::Control(
                    "no loop for break/next, jumping to top level".into()
                ))
            );
            assert!(frontend.runtime_mut().take_output().is_empty());
            assert_eq!(frontend.runtime().scopes.len(), 1);
            assert_eq!(frontend.runtime().control.origin, EvaluationOrigin::Global);
            assert!(frontend.runtime().control.returns.is_empty());
            assert!(frontend.runtime().control.loops.is_empty());
        }
    }
}
