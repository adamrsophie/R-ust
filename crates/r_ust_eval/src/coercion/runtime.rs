use crate::options::{atomic_data, option_integer};
use crate::{EvalError, OutputEvent, RuntimeState, Sexp};

impl RuntimeState {
    pub(crate) fn option_integer_with_warning(
        &mut self,
        value: &Sexp,
    ) -> Result<Option<i32>, EvalError> {
        let integer = option_integer(value);
        if integer.is_none() {
            let warning = match atomic_data(value) {
                Sexp::String(values)
                    if values
                        .first()
                        .and_then(|value| value.as_deref())
                        .is_some_and(|value| !value.trim().is_empty()) =>
                {
                    Some("NAs introduced by coercion")
                }
                Sexp::Real(values) if values.first().is_some_and(|value| !value.is_nan()) => {
                    Some("NAs introduced by coercion to integer range")
                }
                _ => None,
            };
            if let Some(warning) = warning {
                self.emit(OutputEvent::Stderr(format!("Warning: {warning}\n")))?;
            }
        }
        Ok(integer)
    }
}
