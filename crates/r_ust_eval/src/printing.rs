mod options;

use crate::{EvalError, EvaluatedArgument, OutputEvent, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn print_default(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let [value, controls, missings] = arguments else {
            return Err(EvalError::Arity(format!(
                "{} arguments passed to 'print.default' which requires 3",
                arguments.len()
            )));
        };
        let control_value = crate::ordinary_values::read(&controls.value)?;
        let Sexp::PairList(controls) = control_value.as_ref() else {
            return Err(EvalError::Type(
                "invalid print.default control pairlist".into(),
            ));
        };
        let Sexp::Logical(missings) = &missings.value else {
            return Err(EvalError::Type(
                "invalid print.default missing flags".into(),
            ));
        };
        if missings.len() != 8 || missings.iter().any(Option::is_none) {
            return Err(EvalError::Type(
                "invalid print.default missing flags".into(),
            ));
        }
        let quote = options::validate(controls.as_slice())?;
        let text = crate::display::character::format(&value.value, quote)?;
        self.emit(OutputEvent::Stdout(format!("{text}\n")))?;
        self.visible = false;
        Ok(value.value.clone())
    }
}
