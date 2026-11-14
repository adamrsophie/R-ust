use r_ust_sexp::{canonical_language, LanguageError};

use crate::{CallArgument, EvalError, EvalFlow, RuntimeState};

#[cfg(test)]
mod tests;

impl RuntimeState {
    pub(super) fn eval_quote(&mut self, arguments: &[CallArgument]) -> Result<EvalFlow, EvalError> {
        let [argument] = arguments else {
            return Err(EvalError::Arity(format!(
                "{} arguments passed to 'quote' which requires 1",
                arguments.len()
            )));
        };
        if let Some(name) = argument.name.as_deref() {
            if !"expr".starts_with(name) {
                return Err(EvalError::Arity(format!(
                    "supplied argument name '{name}' does not match 'expr'"
                )));
            }
            if !name.is_empty() && name != "expr" {
                self.warn_partial_argument(name, "expr")?;
            }
        }
        let value = canonical_language(&argument.value).map_err(|error| match error {
            LanguageError::InternalCall => EvalError::NotImplemented("quoting internal dispatch"),
            LanguageError::RuntimeValue => EvalError::NotImplemented("quoting runtime handles"),
        })?;
        Ok(EvalFlow::Value(value))
    }
}
