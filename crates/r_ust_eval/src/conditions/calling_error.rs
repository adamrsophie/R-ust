use crate::environment::ApplyFunction;
use crate::{CallArgument, EvalError, EvalFlow, RuntimeState, Sexp};
use r_ust_sexp::{EnvironmentRole, Force};

impl RuntimeState {
    pub(super) fn call_simple_error_handler(
        &mut self,
        expression: &Sexp,
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        self.call_base_condition_hook(expression, (2, call))
    }

    pub(crate) fn call_base_condition_hook(
        &mut self,
        expression: &Sexp,
        cached_call: (usize, &Sexp),
    ) -> Result<EvalFlow, EvalError> {
        crate::base::initialize(&self.current)?;
        let base = self.global.session_root(EnvironmentRole::Base)?;
        let previous = std::mem::replace(&mut self.current, base);
        let result = self.invoke_condition_hook(expression, cached_call);
        let result = self.signal_fresh_result(result);
        self.current = previous;
        result
    }

    fn invoke_condition_hook(
        &mut self,
        expression: &Sexp,
        cached_call: (usize, &Sexp),
    ) -> Result<EvalFlow, EvalError> {
        crate::character_attributes::validate(expression)?;
        self.visible = true;
        let Sexp::Call {
            function,
            arguments,
        } = expression
        else {
            return Err(EvalError::Control("invalid condition helper call".into()));
        };
        let Sexp::Symbol(name) = function.as_ref() else {
            return Err(EvalError::Control("invalid condition helper name".into()));
        };
        let resolved = self.require_source_function(name, false, expression)?;
        let ApplyFunction::Closure(callable) = resolved else {
            return self.call_resolved(resolved, arguments, expression);
        };
        let invocation = self.prepare_closure_call(callable, expression.clone(), arguments)?;
        let Some(CallArgument {
            value: Sexp::Promise { state },
            ..
        }) = invocation.arguments.get(cached_call.0)
        else {
            return Err(EvalError::Control("missing condition call promise".into()));
        };
        match state.begin_force()? {
            Force::Evaluate(lease) => lease.complete(cached_call.1)?,
            Force::Cached(_) => {
                return Err(EvalError::Control(
                    "new condition call promise already forced".into(),
                ))
            }
        }
        self.invoke_closure(invocation)
    }
}
