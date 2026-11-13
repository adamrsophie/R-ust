use crate::{CallArgument, EvalError, RuntimeState, Sexp};
use r_ust_sexp::InternalOperation;

pub fn resolve(name: &str) -> Option<InternalOperation> {
    match name {
        "vector" => Some(InternalOperation::Vector),
        "stop" => Some(InternalOperation::Stop),
        "inherits" => Some(InternalOperation::Inherits),
        "gettext" => Some(InternalOperation::Gettext),
        ".signalCondition" => Some(InternalOperation::SignalCondition),
        ".dfltStop" => Some(InternalOperation::DefaultStop),
        _ => None,
    }
}

impl RuntimeState {
    pub(crate) fn eval_internal_call(
        &mut self,
        arguments: &[CallArgument],
    ) -> Result<Sexp, EvalError> {
        let [argument] = arguments else {
            return Err(EvalError::Arity(format!(
                "{} arguments passed to '.Internal' which requires 1",
                arguments.len()
            )));
        };
        if matches!(argument.value, Sexp::Parenthesized(_)) {
            return Err(EvalError::Domain(
                "there is no .Internal function '('".into(),
            ));
        }
        let Sexp::Call {
            function,
            arguments,
        } = &argument.value
        else {
            return Err(EvalError::Domain("invalid .Internal() argument".into()));
        };
        let Sexp::Symbol(name) = function.as_ref() else {
            return Err(EvalError::Domain("invalid .Internal() argument".into()));
        };
        let operation = resolve(name)
            .ok_or_else(|| EvalError::Domain(format!("there is no .Internal function '{name}'")))?;
        self.eval_internal(operation, arguments)
    }
}
