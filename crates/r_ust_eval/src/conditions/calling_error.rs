use crate::environment::ApplyFunction;
use crate::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp};
use r_ust_sexp::{EnvironmentRole, Force};

impl RuntimeState {
    pub(super) fn call_simple_error_handler(
        &mut self,
        expression: &Sexp,
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let base = self.global.session_root(EnvironmentRole::Base)?;
        let previous = std::mem::replace(&mut self.current, base);
        let result = self.invoke_simple_error_handler(expression, call);
        let result = self.signal_fresh_result(result);
        self.current = previous;
        result
    }

    fn invoke_simple_error_handler(
        &mut self,
        expression: &Sexp,
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        crate::character_attributes::validate(expression)?;
        self.visible = true;
        let Sexp::Call { arguments, .. } = expression else {
            return Err(EvalError::Control(
                "invalid simple error helper call".into(),
            ));
        };
        let resolved = self.require_source_function(".handleSimpleError", false, expression)?;
        let ApplyFunction::Closure(callable) = resolved else {
            return Err(EvalError::NotImplemented(
                "nonclosure .handleSimpleError condition bridge",
            ));
        };
        let invocation = self.prepare_closure_call(callable, expression.clone(), arguments)?;
        let Some(CallArgument {
            value: Sexp::Promise { state },
            ..
        }) = invocation.arguments.get(2)
        else {
            return Err(EvalError::Control("missing condition call promise".into()));
        };
        match state.begin_force()? {
            Force::Evaluate(lease) => lease.complete(call)?,
            Force::Cached(_) => {
                return Err(EvalError::Control(
                    "new condition call promise already forced".into(),
                ))
            }
        }
        self.invoke_closure(invocation)
    }
}
