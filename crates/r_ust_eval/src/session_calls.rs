use super::options::option_query;
use super::{
    bind_named_arguments, coerce_logical_values, expand_dots_call_arguments, string_scalar,
    CallArgument, EvalError, EvalFlow, QuitRequest, RuntimeState, SaveAction, Sexp,
};

impl RuntimeState {
    pub fn run_last(&mut self) -> Result<(), EvalError> {
        let last = self.global.get_local(".Last")?;
        if let Some(last @ Sexp::Closure { .. }) = last {
            self.call_value(last, &[])?;
        }
        Ok(())
    }

    pub(super) fn eval_quit_call(
        &mut self,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let bound = bind_named_arguments(&arguments, ["save", "status", "runLast"])?;
        let mut values = [
            Sexp::String(vec![Some("default".to_string())]),
            Sexp::Integer(vec![Some(0)]),
            Sexp::Logical(vec![Some(true)]),
        ];
        for (index, argument) in bound.into_iter().enumerate() {
            if let Some(argument) = argument.filter(|arg| !matches!(arg.value, Sexp::MissingArg(_)))
            {
                values[index] = match self.eval_flow(&argument.value)? {
                    EvalFlow::Value(value) => value,
                };
            }
        }
        let save = match string_scalar(&values[0], "save")?.as_str() {
            "default" => SaveAction::Default,
            "no" => SaveAction::No,
            "yes" => SaveAction::Yes,
            "ask" => SaveAction::Ask,
            _ => {
                return Err(EvalError::Domain(
                    "unrecognized value of 'save'".to_string(),
                ))
            }
        };
        let status = self.scalar_integer_with_warning(&values[1])?;
        let run_last = coerce_logical_values(&values[2])?
            .first()
            .copied()
            .flatten();
        for (valid, message) in [
            (status.is_some(), "invalid 'status', 0 assumed"),
            (run_last.is_some(), "invalid 'runLast', FALSE assumed"),
        ] {
            if !valid {
                self.record_parse_warnings(&[message.into()])?;
            }
        }
        Err(EvalError::Quit(QuitRequest {
            save,
            status: status.unwrap_or(0),
            run_last: run_last.unwrap_or(false),
        }))
    }

    pub(super) fn eval_get_option(
        &mut self,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        let arguments = expand_dots_call_arguments(arguments, &self.current)?;
        let [x, default] = bind_named_arguments(&arguments, ["x", "default"])?;
        let x = x.ok_or_else(|| EvalError::MissingArgument("x".into()))?;
        let EvalFlow::Value(value) = self.eval_flow(&x.value)?;
        let name = option_query(&value, true)?;
        if let Some(value) = self.get_option(name) {
            let value = value.clone();
            self.visible = true;
            return Ok(EvalFlow::Value(value));
        }
        if let Some(default) = default.filter(|arg| !matches!(arg.value, Sexp::MissingArg(_))) {
            return self.eval_flow(&default.value);
        }
        self.visible = true;
        Ok(EvalFlow::Value(Sexp::Nil))
    }
}
