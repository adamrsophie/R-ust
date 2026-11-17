use super::lookup::MethodEnvironments;
use crate::closure_calls::ClosureInvocation;
use crate::{
    CallArgument, EvalError, EvalFlow, EvaluatedArgument, ListElement, RuntimeState, Sexp,
};
use r_ust_sexp::{EnvironmentRole, Force, PromiseState};

impl RuntimeState {
    pub(crate) fn dispatch_internal_generic(
        &mut self,
        generic: &str,
        arguments: &[EvaluatedArgument],
        call: &Sexp,
    ) -> Result<Option<Sexp>, EvalError> {
        let Some(first) = arguments.first() else {
            return Ok(None);
        };
        let attrs = crate::list_attributes::engine::stored(&first.value)?;
        if !attrs
            .iter()
            .any(|attr| attr.name.as_deref() == Some("class"))
        {
            return Ok(None);
        }
        let classes = super::classes::classes(&first.value)?;
        let values = crate::coercion::storage::read(&classes)?;
        let names = values
            .character_values()
            .ok_or_else(|| EvalError::Type("invalid class attribute".into()))?;
        let environments = MethodEnvironments {
            call: self.current.clone(),
            definition: self.current.session_root(EnvironmentRole::BaseNamespace)?,
        };
        for (index, class) in names.iter().enumerate() {
            let method = format!("{generic}.{}", class.as_deref().unwrap_or("NA"));
            let Some(callable) = self.lookup_method(&method, &environments)? else {
                continue;
            };
            let mut supplied = Vec::new();
            for (position, argument) in arguments.iter().enumerate() {
                let expression = match call {
                    Sexp::Call {
                        function,
                        arguments: syntax,
                    } if matches!(function.as_ref(), Sexp::Symbol(name) if name == generic)
                        && syntax.len() == arguments.len() =>
                    {
                        &syntax[position].value
                    }
                    _ => &argument.value,
                };
                let state = PromiseState::new(
                    expression,
                    &self.current,
                    self.control.origin.clone(),
                    true,
                )?;
                let Force::Evaluate(lease) = state.begin_force()? else {
                    return Err(EvalError::Control(
                        "new dispatch promise already forced".into(),
                    ));
                };
                lease.complete(&argument.value)?;
                supplied.push(CallArgument {
                    name: argument.name.clone(),
                    value: Sexp::Promise { state },
                });
            }
            let bindings = [
                (".Generic", Sexp::String(vec![Some(generic.into())])),
                (".Class", super::classes::suffix(&classes, index)?),
                (".Method", Sexp::String(vec![Some(method)])),
                (".Group", Sexp::String(vec![Some(String::new())])),
                (
                    ".GenericCallEnv",
                    Sexp::Environment(environments.call.clone()),
                ),
                (
                    ".GenericDefEnv",
                    Sexp::Environment(environments.definition.clone()),
                ),
            ]
            .into_iter()
            .map(|(name, value)| ListElement {
                name: Some(name.into()),
                value,
            })
            .collect();
            let EvalFlow::Value(value) = self.invoke_closure(ClosureInvocation {
                callable,
                arguments: supplied,
                call: call.clone(),
                sysparent: self.current.clone(),
                bindings,
            })?;
            return Ok(Some(value));
        }
        Ok(None)
    }
}
