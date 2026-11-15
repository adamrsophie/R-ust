use crate::{
    CallArgument, EvalError, EvalFlow, EvaluatedArgument, RuntimeState, Sexp, SourceFault,
};
use r_ust_sexp::Environment;

impl RuntimeState {
    pub(super) fn internal_do_call(
        &mut self,
        arguments: &[EvaluatedArgument],
    ) -> Result<Sexp, EvalError> {
        let call = self.source_current_call();
        let function = match &arguments[0].value {
            value @ (Sexp::Closure { .. } | Sexp::Primitive(_)) => value.clone(),
            value if value.character_values().is_some() => {
                let Some([name]) = value.character_values() else {
                    return Err(SourceFault::domain(
                        "'what' must be a function or character string".into(),
                        &call,
                    ));
                };
                let name = name.as_deref().unwrap_or("NA");
                if name.is_empty() {
                    return Err(SourceFault::domain(
                        "attempt to use zero-length variable name".into(),
                        &call,
                    ));
                }
                Sexp::Symbol(name.into())
            }
            _ => {
                return Err(SourceFault::domain(
                    "'what' must be a function or character string".into(),
                    &call,
                ))
            }
        };
        if matches!(&function, Sexp::Symbol(name) if name == ".Internal")
            || matches!(
                &function,
                Sexp::Primitive(r_ust_sexp::PrimitiveId::Internal)
            )
        {
            return Err(SourceFault::domain("illegal usage".into(), &call));
        }
        let projection = crate::ordinary_values::read(&arguments[1].value)?;
        let (values, names_present) = if matches!(projection.as_ref(), Sexp::Nil) {
            (&[][..], false)
        } else {
            let view = projection
                .as_ref()
                .ordinary_list()
                .ok_or_else(|| SourceFault::domain("'args' must be a list".into(), &call))?;
            (view.elements, view.names_present)
        };
        let Sexp::Environment(environment) = &arguments[2].value else {
            return Err(SourceFault::domain(
                "'envir' must be an environment".into(),
                &call,
            ));
        };
        let expression = Sexp::Call {
            function: Box::new(function),
            arguments: values
                .iter()
                .map(|element| CallArgument {
                    name: names_present
                        .then(|| element.name.clone().unwrap_or_else(|| "NA".into()))
                        .filter(|name| !name.is_empty()),
                    value: element.value.clone(),
                })
                .collect(),
        };
        let EvalFlow::Value(value) = self.eval_call_in_environment(&expression, environment)?;
        Ok(value)
    }

    pub(crate) fn eval_call_in_environment(
        &mut self,
        expression: &Sexp,
        environment: &Environment,
    ) -> Result<EvalFlow, EvalError> {
        self.global
            .validate_import(&Sexp::Environment(environment.clone()))?;
        self.global.validate_import(expression)?;
        let previous = std::mem::replace(&mut self.current, environment.clone());
        let result = self.eval_flow(expression);
        self.current = previous;
        result
    }
}
