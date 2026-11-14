use crate::{EvalError, RuntimeState, Sexp};
use r_ust_sexp::canonical_language;

pub(super) fn source(value: &Sexp) -> Result<Sexp, EvalError> {
    canonical_language(value)
        .map_err(|_| EvalError::NotImplemented("function body source representation"))
}

impl RuntimeState {
    pub(super) fn body(&mut self, value: &Sexp) -> Result<Sexp, EvalError> {
        match value {
            Sexp::Closure { body, .. } => source(body),
            Sexp::Primitive(_) => Ok(Sexp::Nil),
            _ => {
                self.record_parse_warnings(&["argument is not a function".into()])?;
                Ok(Sexp::Nil)
            }
        }
    }
}
