use super::{atomic, lists, strings::BlankPolicy};
use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn scalar_real_with_warning(&mut self, value: &Sexp) -> Result<f64, EvalError> {
        let (number, warnings) = atomic::scalar_real(value, BlankPolicy::environment());
        self.record_warnings_at(&warnings.messages(), Some(&self.source_current_call()))?;
        Ok(number)
    }

    pub(crate) fn scalar_integer_with_warning(
        &mut self,
        value: &Sexp,
    ) -> Result<Option<i32>, EvalError> {
        let (integer, warnings) = atomic::scalar_integer(value, BlankPolicy::environment());
        self.record_warnings_at(&warnings.messages(), Some(&self.source_current_call()))?;
        Ok(integer)
    }

    pub(crate) fn call_numeric_coercion(
        &mut self,
        name: &str,
        args: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let integer = name == "as.integer";
        let Some(first) = args.first() else {
            return Ok(if integer {
                Sexp::Integer(Vec::new())
            } else {
                Sexp::Real(Vec::new())
            });
        };
        if let Some(name) = first.name.as_deref().filter(|name| *name != "x") {
            return Err(EvalError::Arity(format!(
                "supplied argument name '{name}' does not match 'x'"
            )));
        }
        if matches!(first.value, Sexp::MissingArg(_)) {
            return Err(EvalError::Arity("argument 1 is empty".into()));
        }
        lists::convert(
            &first.value,
            (integer, BlankPolicy::environment()),
            |warnings| self.record_parse_warnings(&warnings.messages()),
        )
    }
}
