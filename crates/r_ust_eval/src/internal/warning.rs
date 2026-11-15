use crate::{EvalError, EvaluatedArgument, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn internal_warning(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let call = self.source_stop_call(super::as_logical(&arguments[0].value).unwrap_or(true));
        let value = if matches!(arguments[3].value, Sexp::Nil) {
            Sexp::Nil
        } else {
            Sexp::String(super::character_values(&arguments[3].value)?)
        };
        let message = if matches!(value, Sexp::Nil) {
            ""
        } else {
            value
                .character_values()
                .and_then(|values| values.first())
                .map(|value| value.as_deref().unwrap_or("NA"))
                .unwrap_or(" [invalid string in warning(.)]")
        };
        self.conditions.immediate_warning = super::as_logical(&arguments[1].value).unwrap_or(true);
        self.conditions.no_break_warning = super::as_logical(&arguments[2].value).unwrap_or(true);
        let result = self.signal_simple_warning(message, &call);
        self.conditions.immediate_warning = false;
        self.conditions.no_break_warning = false;
        result?;
        Ok(value)
    }
}
