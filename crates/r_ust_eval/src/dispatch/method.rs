use super::lookup::MethodEnvironments;
use crate::closure_calls::ClosureInvocation;
use crate::dynamic_context::{Context, ContextKind};
use crate::{ControlTransfer, EvalError, EvalFlow, ListElement, RuntimeState, Sexp};

pub(super) struct SelectedMethod {
    pub(super) callable: Sexp,
    pub(super) name: String,
    pub(super) generic: String,
    pub(super) class: Sexp,
    pub(super) environments: MethodEnvironments,
}

impl RuntimeState {
    pub(super) fn dispatch_method(
        &mut self,
        context: Context,
        method: SelectedMethod,
    ) -> Result<EvalFlow, EvalError> {
        let ContextKind::Return(target) = context.kind else {
            return Err(EvalError::Domain(
                "'UseMethod' used in an inappropriate fashion".into(),
            ));
        };
        let call = match context.call {
            Sexp::Call { arguments, .. } => Sexp::Call {
                function: Box::new(Sexp::Symbol(method.name.clone())),
                arguments,
            },
            Sexp::Nil => Sexp::Nil,
            _ => return Err(EvalError::Type("invalid generic call metadata".into())),
        };
        let bindings = [
            (".Generic", Sexp::String(vec![Some(method.generic)])),
            (".Class", method.class),
            (".Method", Sexp::String(vec![Some(method.name)])),
            (".Group", Sexp::String(vec![Some(String::new())])),
            (
                ".GenericCallEnv",
                Sexp::Environment(method.environments.call),
            ),
            (
                ".GenericDefEnv",
                Sexp::Environment(method.environments.definition),
            ),
        ]
        .into_iter()
        .map(|(name, value)| ListElement {
            name: Some(name.into()),
            value,
        })
        .collect();
        let Some(active) = self.control.contexts.last_mut() else {
            return Err(EvalError::Type("missing generic context".into()));
        };
        let saved = active.generic;
        active.generic = true;
        let result = match &method.callable {
            Sexp::Closure { .. } => self.invoke_closure(ClosureInvocation {
                callable: method.callable,
                arguments: context.arguments,
                call,
                sysparent: context.sysparent,
                bindings,
            }),
            Sexp::Primitive(_) => self.call_value(method.callable, &context.arguments, &call),
            _ => Err(EvalError::Type("attempt to apply non-function".into())),
        };
        if let Some(active) = self.control.contexts.last_mut() {
            active.generic = saved;
        }
        let EvalFlow::Value(value) = result?;
        Err(EvalError::Transfer(ControlTransfer::Return(target, value)))
    }
}
