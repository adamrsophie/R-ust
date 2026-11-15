use crate::environment::{has_builtin, ApplyFunction};
use crate::{EvalError, EvalFlow, RuntimeState, Sexp, SourceFault};
use r_ust_sexp::{PrimitiveId, SyntaxPrimitive};

impl RuntimeState {
    pub(super) fn resolve_evaluated_head(
        &mut self,
        expression: &Sexp,
        call: &Sexp,
    ) -> Result<ApplyFunction, EvalError> {
        let value = if let Sexp::Call {
            function,
            arguments,
        } = expression
        {
            let selected = if let Sexp::Symbol(name) = function.as_ref() {
                self.require_source_function(name, has_builtin(name), expression)?
            } else {
                self.resolve_evaluated_head(function, expression)?
            };
            if matches!(
                selected,
                ApplyFunction::Primitive(PrimitiveId::Syntax(SyntaxPrimitive::Parenthesis))
            ) {
                if let [argument] = arguments.as_slice() {
                    return self.resolve_parenthesized_function(&argument.value);
                }
            }
            let EvalFlow::Value(value) = self.call_resolved(selected, arguments, expression)?;
            value
        } else {
            let EvalFlow::Value(value) = self.eval_flow(expression)?;
            value
        };
        match value {
            Sexp::Primitive(id) => Ok(ApplyFunction::Primitive(id)),
            closure @ Sexp::Closure { .. } => Ok(ApplyFunction::Closure(closure)),
            _ => Err(SourceFault::type_error(
                "attempt to apply non-function".into(),
                call,
            )),
        }
    }
}
