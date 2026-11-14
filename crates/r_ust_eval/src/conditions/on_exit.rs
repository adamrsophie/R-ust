use crate::{
    dynamic_context::ContextKind, CallArgument, EvalError, EvalFlow, FunctionParameter,
    RuntimeState, Sexp,
};

impl RuntimeState {
    pub(crate) fn eval_on_exit(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let parameters = ["expr", "add", "after"].map(FunctionParameter::required);
        let environment = self.current.clone();
        let mut matched = self.bind_promised_arguments(&parameters, arguments, Some(call))?;
        let expression = match matched.remove("expr") {
            Some(Sexp::MissingArg(_) | Sexp::MissingValue) | None => Sexp::Nil,
            Some(value) => value,
        };
        let add = self.cleanup_flag(matched.remove("add"), (false, "add"), call)?;
        let after = self.cleanup_flag(matched.remove("after"), (true, "lifo"), call)?;
        if let Some(context) = self.control.contexts.iter_mut().rev().find(|context| {
            matches!(context.kind, ContextKind::Return(_)) && context.execution == environment
        }) {
            if !add {
                context.cleanup.clear();
            }
            if add || !matches!(expression, Sexp::Nil) {
                if after {
                    context.cleanup.push_back(expression);
                } else {
                    context.cleanup.push_front(expression);
                }
            }
        }
        self.visible = false;
        Ok(EvalFlow::Value(Sexp::Nil))
    }

    fn cleanup_flag(
        &mut self,
        value: Option<Sexp>,
        default: (bool, &str),
        call: &Sexp,
    ) -> Result<bool, EvalError> {
        let Some(value) = value else {
            return Ok(default.0);
        };
        if matches!(value, Sexp::MissingArg(_) | Sexp::MissingValue) {
            return Ok(default.0);
        }
        let value = self.eval_regular_argument(&value)?;
        crate::internal::as_logical(&value).ok_or_else(|| {
            crate::SourceFault::domain(format!("invalid '{}' argument", default.1), call)
        })
    }
}
