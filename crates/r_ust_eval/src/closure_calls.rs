use super::{
    expand_dots_call_arguments, promise, CallArgument, ClosureEnvironment, Environment, EvalError,
    EvalFlow, FunctionParameter, RuntimeState, Sexp,
};

impl RuntimeState {
    pub(super) fn call_closure(
        &mut self,
        parameters: &[FunctionParameter],
        body: &Sexp,
        environment: &ClosureEnvironment,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        let caller = self.current.clone();
        let arguments = expand_dots_call_arguments(arguments, &caller)?;
        let bindings = self.bind_supplied_argument_promises(parameters, &arguments, &caller)?;
        let frame = match environment {
            ClosureEnvironment::Captured(frame) => frame.child()?,
            ClosureEnvironment::Base => Environment::global(),
        };
        for (name, value) in bindings {
            frame.insert(name, &value)?;
        }
        let (id, previous) = self.control.enter_return_context()?;
        let saved = std::mem::replace(&mut self.current, frame);
        let result = self
            .bind_missing_default_promises(parameters)
            .and_then(|()| self.eval_flow(body));
        self.current = saved;
        self.control.leave_return_context(id, previous, result)
    }

    fn bind_missing_default_promises(
        &mut self,
        parameters: &[FunctionParameter],
    ) -> Result<(), EvalError> {
        for parameter in parameters {
            if self.current.contains_local(&parameter.name)?
                && !matches!(
                    self.raw_lookup(&parameter.name)?,
                    Some(Sexp::MissingArg(_) | Sexp::MissingValue)
                )
            {
                continue;
            }
            let value = match &parameter.default {
                Some(default) => promise(
                    default.clone(),
                    self.current.clone(),
                    self.control.origin.clone(),
                    false,
                )?,
                None => Sexp::MissingArg(parameter.name.clone()),
            };
            self.assign_local(parameter.name.clone(), value)?;
        }
        Ok(())
    }
}
