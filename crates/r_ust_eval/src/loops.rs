use super::control::{catch_loop, LoopFlow};
use super::{condition_truth, sequence_elements, EvalError, EvalFlow, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_for(
        &mut self,
        variable: &str,
        sequence: &Sexp,
        body: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let EvalFlow::Value(sequence) = self.eval_flow(sequence)?;
        let values = sequence_elements(&sequence)?;
        self.with_loop(|runtime, id| {
            for value in values {
                runtime.assign_local(variable.into(), value)?;
                match catch_loop(id, runtime.eval_flow(body))? {
                    LoopFlow::Value(_) | LoopFlow::Next => {}
                    LoopFlow::Break => break,
                }
            }
            Ok(EvalFlow::Value(Sexp::Nil))
        })
    }

    pub(super) fn eval_while(
        &mut self,
        condition: &Sexp,
        body: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        self.with_loop(|runtime, id| {
            loop {
                match catch_loop(id, runtime.eval_flow(condition))? {
                    LoopFlow::Value(value) if condition_truth(&value)? => {}
                    LoopFlow::Value(_) | LoopFlow::Break => break,
                    LoopFlow::Next => continue,
                }
                match catch_loop(id, runtime.eval_flow(body))? {
                    LoopFlow::Value(_) | LoopFlow::Next => {}
                    LoopFlow::Break => break,
                }
            }
            Ok(EvalFlow::Value(Sexp::Nil))
        })
    }

    pub(super) fn eval_repeat(&mut self, body: &Sexp) -> Result<EvalFlow, EvalError> {
        self.with_loop(|runtime, id| {
            while let LoopFlow::Value(_) | LoopFlow::Next = catch_loop(id, runtime.eval_flow(body))?
            {
            }
            Ok(EvalFlow::Value(Sexp::Nil))
        })
    }
}
