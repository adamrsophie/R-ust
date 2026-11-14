use super::{
    expand_dots_call_arguments, CallArgument, ClosureEnvironment, EvalError, EvalFlow,
    FunctionParameter, ListElement, RuntimeState, Sexp,
};
use r_ust_sexp::Environment;

pub(super) struct ClosureInvocation {
    pub(super) callable: Sexp,
    pub(super) arguments: Vec<CallArgument>,
    pub(super) call: Sexp,
    pub(super) sysparent: Environment,
    pub(super) bindings: Vec<ListElement>,
}

impl RuntimeState {
    pub(super) fn prepare_closure_call(
        &mut self,
        callable: Sexp,
        call: Sexp,
        arguments: &[CallArgument],
    ) -> Result<ClosureInvocation, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let arguments = arguments
            .into_iter()
            .map(|argument| {
                let value = match argument.value {
                    value @ (Sexp::MissingArg(_) | Sexp::MissingValue) => value,
                    value => self.promise(value, true)?,
                };
                Ok(CallArgument {
                    name: argument.name,
                    value,
                })
            })
            .collect::<Result<Vec<_>, EvalError>>()?;
        Ok(ClosureInvocation {
            callable,
            arguments,
            call,
            sysparent: self.current.clone(),
            bindings: Vec::new(),
        })
    }

    pub(super) fn invoke_closure(
        &mut self,
        invocation: ClosureInvocation,
    ) -> Result<EvalFlow, EvalError> {
        self.with_closure_context(invocation, |runtime, body| runtime.eval_flow(body))
    }

    pub(super) fn with_closure_context(
        &mut self,
        invocation: ClosureInvocation,
        evaluate: impl FnOnce(&mut Self, &Sexp) -> Result<EvalFlow, EvalError>,
    ) -> Result<EvalFlow, EvalError> {
        let Sexp::Closure {
            parameters,
            body,
            environment,
        } = &invocation.callable
        else {
            return Err(EvalError::Type("attempt to apply non-function".into()));
        };
        let bindings = self.bind_promised_arguments(
            parameters,
            &invocation.arguments,
            Some(&invocation.call),
        )?;
        let frame = match environment {
            ClosureEnvironment::Captured(frame) => frame.child()?,
            ClosureEnvironment::Base => self
                .global
                .session_root(r_ust_sexp::EnvironmentRole::BaseNamespace)?
                .child()?,
        };
        for (name, value) in bindings {
            frame.insert(name, &value)?;
        }
        self.with_return_context(
            super::dynamic_context::ReturnContext {
                execution: frame,
                sysparent: invocation.sysparent,
                callable: invocation.callable.clone(),
                arguments: invocation.arguments,
                call: invocation.call,
                generic: false,
            },
            |runtime| {
                runtime.bind_missing_default_promises(parameters)?;
                for binding in invocation.bindings {
                    if let Some(name) = binding.name {
                        if !runtime.current.contains_local(&name)? {
                            runtime.assign_local(name, binding.value)?;
                        }
                    }
                }
                evaluate(runtime, body)
            },
        )
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
                Some(default) => self.promise(default.clone(), false)?,
                None => Sexp::MissingArg(parameter.name.clone()),
            };
            self.assign_local(parameter.name.clone(), value)?;
        }
        Ok(())
    }
}
