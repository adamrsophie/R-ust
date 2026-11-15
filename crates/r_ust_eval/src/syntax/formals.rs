use crate::{
    CallArgument, ClosureEnvironment, EvalError, EvalFlow, FunctionParameter, RuntimeState, Sexp,
    SourceFault,
};

impl RuntimeState {
    pub(super) fn syntax_function(
        &self,
        arguments: &[CallArgument],
        call: &Sexp,
    ) -> Result<EvalFlow, EvalError> {
        let [formals, body, ..] = arguments else {
            return Err(SourceFault::arity(
                "incorrect number of arguments to \"function\"".into(),
                call,
            ));
        };
        let values = match &formals.value {
            Sexp::Nil => Vec::new(),
            Sexp::PairList(values) => values.as_slice().to_vec(),
            Sexp::PairListObject(values) => values.elements()?,
            _ => return Err(invalid_formals(call)),
        };
        let parameters = values
            .iter()
            .map(|value| {
                let name = value
                    .name
                    .as_ref()
                    .filter(|name| !name.is_empty())
                    .ok_or_else(|| invalid_formals(call))?;
                Ok(FunctionParameter {
                    name: name.clone(),
                    default: match value.value {
                        Sexp::MissingValue | Sexp::MissingArg(_) => None,
                        _ => Some(value.value.clone()),
                    },
                })
            })
            .collect::<Result<Vec<_>, EvalError>>()?;
        if matches!(
            body.value,
            Sexp::Closure { .. } | Sexp::Primitive(_) | Sexp::Dots(_)
        ) {
            return Err(SourceFault::type_error(
                "invalid body argument for 'function'".into(),
                call,
            ));
        }
        Ok(EvalFlow::Value(Sexp::Closure {
            attributes: arguments
                .get(2)
                .filter(|source| !matches!(source.value, Sexp::Nil))
                .map(|source| {
                    vec![crate::ListElement {
                        name: Some("srcref".into()),
                        value: source.value.clone(),
                    }]
                })
                .unwrap_or_default(),
            parameters,
            body: Box::new(match &body.value {
                Sexp::MissingValue => Sexp::MissingArg(String::new()),
                value => value.clone(),
            }),
            environment: ClosureEnvironment::Captured(self.current.clone()),
        }))
    }
}

fn invalid_formals(call: &Sexp) -> EvalError {
    SourceFault::type_error("invalid formal argument list for \"function\"".into(), call)
}
