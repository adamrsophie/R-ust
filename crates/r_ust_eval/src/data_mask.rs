use super::dynamic_context::ReturnContext;
use super::{data_mask_scope, EvalError, EvalFlow, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_in_data_mask(
        &mut self,
        data: &Sexp,
        expression: &Sexp,
    ) -> Result<Sexp, EvalError> {
        let scope = data_mask_scope(data)?;
        let frame = self.current.child()?;
        for (name, value) in scope {
            frame.insert(name, &value)?;
        }
        // shortcut: this data-mask path has no internal-eval metadata, replace with genuine eval when ported.
        let EvalFlow::Value(value) = self.eval_selected(
            ReturnContext {
                execution: frame,
                sysparent: self.current.clone(),
                callable: Sexp::Nil,
                arguments: Vec::new(),
                call: Sexp::Nil,
                generic: false,
            },
            expression,
        )?;
        Ok(value)
    }
}
