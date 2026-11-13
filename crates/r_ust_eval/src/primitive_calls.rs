use r_ust_sexp::PrimitiveId;

use super::{
    is_missing_binding, ApplyFunction, CallArgument, EvalError, EvalFlow, EvaluatedArgument,
    RuntimeState, Sexp,
};

impl RuntimeState {
    pub(super) fn eval_apply_extra(
        &mut self,
        function: &ApplyFunction,
        arguments: &[CallArgument],
    ) -> Result<Vec<EvaluatedArgument>, EvalError> {
        if matches!(
            function,
            ApplyFunction::Primitive(PrimitiveId::Return | PrimitiveId::Missing)
        ) {
            return Ok(Vec::new());
        }
        arguments
            .iter()
            .map(|argument| {
                Ok(EvaluatedArgument {
                    name: argument.name.clone(),
                    value: self.eval_regular_argument(&argument.value)?,
                })
            })
            .collect()
    }

    fn eval_missing(&self, arguments: &[CallArgument]) -> Result<EvalFlow, EvalError> {
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
            Sexp::String(values) if values.len() == 1 => values[0].as_deref().unwrap_or("NA"),
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

    fn eval_return(&mut self, arguments: &[CallArgument]) -> Result<EvalFlow, EvalError> {
        if arguments.len() > 1 {
            return Err(EvalError::Arity(
                "multi-argument returns are not permitted".into(),
            ));
        }
        let value = if let Some(argument) = arguments.first() {
            let EvalFlow::Value(value) = self.eval_flow(&argument.value)?;
            value
        } else {
            Sexp::Nil
        };
        Err(self.control.return_signal(value))
    }

    pub(super) fn call_value(
        &mut self,
        value: Sexp,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        match value {
            Sexp::Closure {
                parameters,
                body,
                environment,
            } => self.call_closure(&parameters, &body, &environment, arguments),
            Sexp::Primitive(id) => self.call_primitive(id, arguments),
            _ => Err(EvalError::Type("attempt to apply non-function".into())),
        }
    }

    fn call_primitive(
        &mut self,
        id: PrimitiveId,
        arguments: &[CallArgument],
    ) -> Result<EvalFlow, EvalError> {
        self.visible = true;
        match id {
            PrimitiveId::Return => self.eval_return(arguments),
            PrimitiveId::Missing => self.eval_missing(arguments),
            PrimitiveId::Sum | PrimitiveId::Length | PrimitiveId::Invisible => {
                let values = self.eval_arguments(arguments, id == PrimitiveId::Sum)?;
                self.call_eager_primitive(id, &values).map(EvalFlow::Value)
            }
        }
    }

    pub(super) fn call_eager_primitive(
        &mut self,
        id: PrimitiveId,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        match id {
            PrimitiveId::Length | PrimitiveId::Invisible => {
                if arguments.len() != 1 && !(id == PrimitiveId::Invisible && arguments.is_empty()) {
                    return Err(EvalError::Arity(format!(
                        "{} arguments passed to '{}' which requires 1",
                        arguments.len(),
                        id.name()
                    )));
                }
                if let Some(name) = arguments
                    .first()
                    .and_then(|argument| argument.name.as_deref())
                    .filter(|name| *name != "x")
                {
                    return Err(EvalError::Arity(format!(
                        "supplied argument name '{name}' does not match 'x'"
                    )));
                }
                self.call_builtin(id.name(), arguments)
            }
            PrimitiveId::Sum => self.call_builtin(id.name(), arguments),
            PrimitiveId::Return | PrimitiveId::Missing => {
                let arguments = [
                    CallArgument::positional(Sexp::Symbol("X".into())),
                    CallArgument::positional(Sexp::Symbol("...".into())),
                ];
                let EvalFlow::Value(value) = self.call_primitive(id, &arguments)?;
                Ok(value)
            }
        }
    }
}

pub(super) fn format_primitive(id: PrimitiveId) -> String {
    let signature = match id {
        PrimitiveId::Sum => "function (..., na.rm = FALSE)  ",
        PrimitiveId::Length | PrimitiveId::Missing => "function (x)  ",
        PrimitiveId::Invisible => "function (x = NULL)  ",
        PrimitiveId::Return => "",
    };
    format!("{signature}.Primitive(\"{}\")", id.name())
}
