use crate::{is_missing_binding, CallArgument, EvalError, EvalFlow, RuntimeState, Sexp};

impl RuntimeState {
    pub(super) fn eval_missing(&self, arguments: &[CallArgument]) -> Result<EvalFlow, EvalError> {
        if arguments.len() != 1 {
            return Err(EvalError::Arity(format!(
                "{} arguments passed to 'missing' which requires 1",
                arguments.len()
            )));
        }
        if let Some(name) = arguments[0].name.as_deref().filter(|name| *name != "x") {
            return Err(EvalError::Arity(format!(
                "supplied argument name '{name}' does not match 'x'"
            )));
        }
        let name = match &arguments[0].value {
            Sexp::Symbol(name) => name,
            Sexp::String(values) | Sexp::AttributedCharacter { values, .. }
                if values.len() == 1 =>
            {
                values[0].as_deref().unwrap_or("NA")
            }
            _ => return Err(EvalError::Type("invalid use of 'missing'".into())),
        };
        if let Some(index) = name
            .strip_prefix("..")
            .and_then(|value| value.parse::<usize>().ok())
            .and_then(|value| value.checked_sub(1))
        {
            if let Some(Sexp::Dots(values)) = self.current.get_local("...")? {
                let absent = match values.get(index) {
                    Some(value) => is_missing_binding(&value.value)?,
                    None => true,
                };
                return Ok(EvalFlow::Value(Sexp::Logical(vec![Some(absent)])));
            }
        }
        let value = self.current.get_local(name)?.ok_or_else(|| {
            EvalError::Domain(format!("'missing({name})' did not find an argument"))
        })?;
        let absent = match &value {
            Sexp::Dots(values) => values.is_empty(),
            _ => is_missing_binding(&value)?,
        };
        Ok(EvalFlow::Value(Sexp::Logical(vec![Some(absent)])))
    }

    pub(super) fn eval_return(
        &mut self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        if arguments.len() > 1 {
            return Err(EvalError::Arity(
                "multi-argument returns are not permitted".into(),
            ));
        }
        let value = if let Some(argument) = arguments.first() {
            let EvalFlow::Value(value) = self.eval_syntax_operand(&argument.value, call)?;
            value
        } else {
            Sexp::Nil
        };
        Err(self.control.return_signal(&self.current, value))
    }
}
