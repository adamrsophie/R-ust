use crate::{dynamic_context::ContextKind, EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn condition_nargs(
        &mut self,
        arguments: &[EvaluatedArgument],
        call: &Sexp,
    ) -> Result<Sexp, EvalError> {
        if !arguments.is_empty() {
            let suffix = if arguments.len() == 1 { "" } else { "s" };
            return Err(crate::SourceFault::arity(
                format!(
                    "{} argument{suffix} passed to 'nargs' which requires 0",
                    arguments.len()
                ),
                call,
            ));
        }
        let count = self
            .control
            .contexts
            .iter()
            .rev()
            .find(|context| {
                matches!(context.kind, ContextKind::Return(_)) && context.execution == self.current
            })
            .map(|context| i32::try_from(context.arguments.len()))
            .transpose()
            .map_err(|_| EvalError::Domain("too many arguments".into()))?;
        self.visible = true;
        Ok(Sexp::Integer(vec![count]))
    }
}
